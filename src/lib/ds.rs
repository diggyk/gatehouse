#![warn(missing_docs)]

//! The datastore holds all the policies, targets, and internal PIP data

use std::collections::{HashMap, HashSet};
use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::entity::RegisteredEntity;
use crate::msgs::{DsRequest, DsResponse};
use crate::proto::entities::{
    AddEntityRequest, Entity, GetAllEntitiesRequest, ModifyEntityRequest, RemoveEntityRequest,
};
use crate::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};
use crate::storage::file::FileStorage;
use crate::target::RegisteredTarget;

pub struct Datastore {
    rx: flume::Receiver<DsRequest>,
    backend: FileStorage,

    /// HashMap from type string to HashMap of name to registered target
    targets: HashMap<String, HashMap<String, RegisteredTarget>>,

    /// HashMap from type string to HashMap of name to registered entity
    entities: HashMap<String, HashMap<String, RegisteredEntity>>,
}

impl Datastore {
    pub(crate) async fn create() -> flume::Sender<DsRequest> {
        let (tx, rx) = flume::unbounded();
        let backend = FileStorage::new("/tmp/gatehouse").await;

        let targets = backend
            .load_targets()
            .await
            .expect("Could not load targets from backend");

        let entities = backend
            .load_entities()
            .await
            .expect("Could not load entities from backend");

        let mut ds = Datastore {
            rx,
            backend,
            targets,
            entities,
        };

        tokio::spawn(async move {
            ds.run().await;
        });

        tx
    }

    /// Our main run loop.  We listen to incoming messages from the server and respond accordingly
    async fn run(&mut self) {
        while let Ok(msg) = self.rx.recv_async().await {
            match msg {
                DsRequest::AddTarget(req, tx) => self.add_target(req, tx).await,
                DsRequest::ModifyTarget(req, tx) => self.modify_target(req, tx).await,
                DsRequest::RemoveTarget(req, tx) => self.remove_target(req, tx).await,
                DsRequest::GetTargets(req, tx) => self.get_targets(req, tx).await,
                DsRequest::AddEntity(req, tx) => self.add_entity(req, tx).await,
                DsRequest::ModifyEntity(req, tx) => self.modify_entity(req, tx).await,
                DsRequest::RemoveEntity(req, tx) => self.remove_entity(req, tx).await,
                DsRequest::GetEntities(req, tx) => self.get_entities(req, tx).await,
            }
        }
    }

    /// Add a new target
    async fn add_target(&mut self, req: AddTargetRequest, tx: Sender<DsResponse>) {
        // add to the local cache
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // get or create the hashmap for this "type" of target
        let typed_targets = self
            .targets
            .entry(typestr.clone())
            .or_insert_with(HashMap::new);

        // if target already exists, return an error
        if typed_targets.contains_key(&name) {
            println!("Target already exists: {}/{}", typestr, name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Target already exists",
            )));
            return;
        }

        // convert the attributes to a hashmap
        let mut attributes = HashMap::new();
        for attrib in req.attributes {
            attributes.insert(attrib.0, HashSet::from_iter(attrib.1.values));
        }

        let new_target = RegisteredTarget::new(&name, &typestr, req.actions, attributes);

        match self.backend.save_target(new_target.clone()).await {
            Ok(_) => {
                typed_targets.insert(name, new_target.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleTarget(new_target.into()));
    }

    /// Modify and existing target
    async fn modify_target(&mut self, req: ModifyTargetRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        if !self.targets.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by type",
            )));
            return;
        }

        let typed_targets = self.targets.get_mut(&typestr).unwrap();
        if !(typed_targets.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by name",
            )));
            return;
        }

        // we'll work with a clone of the target in case persistence fails
        let mut updated_target = typed_targets.get(&name).unwrap().clone();

        // update actions
        for action in req.add_actions {
            updated_target.actions.insert(action.to_ascii_lowercase());
        }
        for action in req.remove_actions {
            updated_target.actions.remove(&action.to_ascii_lowercase());
        }

        // update attributes
        for attrib in req.add_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            let attrib_entry = updated_target.attributes.entry(key).or_default();
            attrib_entry.extend(values);
        }
        for attrib in req.remove_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            if let Some(current_values) = updated_target.attributes.get_mut(&key) {
                for value in values {
                    current_values.remove(&value);
                }

                if current_values.is_empty() {
                    updated_target.attributes.remove(&key);
                }
            }
        }

        match self.backend.save_target(updated_target.clone()).await {
            Ok(_) => {
                let _ = typed_targets.insert(name.clone(), updated_target.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleTarget(updated_target.clone().into()));
    }

    /// Remove an existing target
    async fn remove_target(&mut self, req: RemoveTargetRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // make sure the target type exists
        if !self.targets.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by type",
            )));
            return;
        }

        // make sure the target exists
        let typed_targets = self.targets.get_mut(&typestr).unwrap();
        if !(typed_targets.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by name",
            )));
            return;
        }

        // get the existing target
        let existing_target = typed_targets.get(&name).unwrap().clone();

        // try to persist the new target to the backend and if that succeeds, update it in memory
        match self.backend.remove_target(existing_target.clone()).await {
            Ok(_) => {
                let _ = typed_targets.remove(&name);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleTarget(existing_target.clone().into()));
    }

    /// Get all targets, optionally filtered by type
    async fn get_targets(&mut self, req: GetAllTargetsRequest, tx: Sender<DsResponse>) {
        let typestr = req.typestr.map(|t| t.to_ascii_lowercase());
        let name = req.name.map(|t| t.to_ascii_lowercase());
        let mut targets: Vec<Target> = Vec::new();

        for typemap in self.targets.iter() {
            if let Some(ref filter_type) = typestr {
                if typemap.0.as_str() != filter_type {
                    continue;
                }
            }
            for target in typemap.1.iter() {
                if let Some(ref name_ref) = name {
                    if target.0.as_str() != name_ref {
                        continue;
                    }
                }
                targets.push(target.1.clone().into());
            }
        }

        let _ = tx.send(DsResponse::MultipleTargets(targets));
    }

    /// Add a new entity
    async fn add_entity(&mut self, req: AddEntityRequest, tx: Sender<DsResponse>) {
        // add to the local cache
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // get or create the hashmap for this "type" of target
        let typed_entities = self
            .entities
            .entry(typestr.clone())
            .or_insert_with(HashMap::new);

        // if entity already exists, return an error
        if typed_entities.contains_key(&name) {
            println!("Entity already exists: {}/{}", typestr, name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Entity already exists",
            )));
            return;
        }

        // convert the attributes to a hashmap
        let mut attributes = HashMap::new();
        for attrib in req.attributes {
            attributes.insert(attrib.0, HashSet::from_iter(attrib.1.values));
        }

        let new_entity = RegisteredEntity::new(&name, &typestr, attributes);

        match self.backend.save_entity(new_entity.clone()).await {
            Ok(_) => {
                typed_entities.insert(name, new_entity.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleEntity(new_entity.into()));
    }

    /// Modify and existing entity
    async fn modify_entity(&mut self, req: ModifyEntityRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        if !self.entities.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find entity by type",
            )));
            return;
        }

        let typed_entities = self.entities.get_mut(&typestr).unwrap();
        if !(typed_entities.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find entity by name",
            )));
            return;
        }

        // we'll work with a clone of the entity in case persistence fails
        let mut updated_entity = typed_entities.get(&name).unwrap().clone();

        // update attributes
        for attrib in req.add_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            let attrib_entry = updated_entity.attributes.entry(key).or_default();
            attrib_entry.extend(values);
        }
        for attrib in req.remove_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            if let Some(current_values) = updated_entity.attributes.get_mut(&key) {
                for value in values {
                    current_values.remove(&value);
                }

                if current_values.is_empty() {
                    updated_entity.attributes.remove(&key);
                }
            }
        }

        match self.backend.save_entity(updated_entity.clone()).await {
            Ok(_) => {
                let _ = typed_entities.insert(name.clone(), updated_entity.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleEntity(updated_entity.clone().into()));
    }

    /// Remove an existing entity
    async fn remove_entity(&mut self, req: RemoveEntityRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // make sure the target type exists
        if !self.entities.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find entity by type",
            )));
            return;
        }

        // make sure the target exists
        let typed_entities = self.entities.get_mut(&typestr).unwrap();
        if !(typed_entities.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find entity by name",
            )));
            return;
        }

        // get the existing entity
        let existing_entity = typed_entities.get(&name).unwrap().clone();

        // try to persist the new target to the backend and if that succeeds, update it in memory
        match self.backend.remove_entity(existing_entity.clone()).await {
            Ok(_) => {
                let _ = typed_entities.remove(&name);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleEntity(existing_entity.clone().into()));
    }

    /// Get all entities, optionally filtered by type
    async fn get_entities(&mut self, req: GetAllEntitiesRequest, tx: Sender<DsResponse>) {
        let typestr = req.typestr.map(|t| t.to_ascii_lowercase());
        let name = req.name.map(|t| t.to_ascii_lowercase());
        let mut entities: Vec<Entity> = Vec::new();

        for typemap in self.entities.iter() {
            if let Some(ref filter_type) = typestr {
                if typemap.0.as_str() != filter_type {
                    continue;
                }
            }
            for entity in typemap.1.iter() {
                if let Some(ref name_ref) = name {
                    if entity.0.as_str() != name_ref {
                        continue;
                    }
                }
                entities.push(entity.1.clone().into());
            }
        }

        let _ = tx.send(DsResponse::MultipleEntities(entities));
    }
}
