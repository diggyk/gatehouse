#![warn(missing_docs)]

//! The datastore holds all the policies, targets, and internal PIP data

use std::collections::HashMap;
use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::msgs::{DsRequest, DsResponse};
use crate::proto::targets::{
    AddTargetActionRequest, AddTargetRequest, GetAllTargetsRequest, RemoveTargetActionRequest,
    RemoveTargetRequest, Target,
};
use crate::storage::file::FileStorage;
use crate::target::RegisteredTarget;

pub struct Datastore {
    rx: flume::Receiver<DsRequest>,
    backend: FileStorage,

    /// HashMap from type string to HashMap of name to registered target
    targets: HashMap<String, HashMap<String, RegisteredTarget>>,
}

impl Datastore {
    pub(crate) async fn create() -> flume::Sender<DsRequest> {
        let (tx, rx) = flume::unbounded();
        let backend = FileStorage::new("/tmp/gatehouse").await;

        let targets = backend
            .load_targets()
            .await
            .expect("Could not load targets from backend");

        let mut ds = Datastore {
            rx,
            backend,
            targets,
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
                DsRequest::AddTarget(req, tx) => {
                    self.add_target(req, tx).await;
                }
                DsRequest::AddTargetActions(req, tx) => self.add_target_action(req, tx).await,
                DsRequest::RemoveTargetActions(req, tx) => self.remove_target_action(req, tx).await,
                DsRequest::RemoveTarget(req, tx) => self.remove_target(req, tx).await,
                DsRequest::GetTargets(req, tx) => self.get_targets(req, tx).await,
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

        let new_target = RegisteredTarget::new(&name, &typestr, req.actions);

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

    /// Add an action to an existing target
    async fn add_target_action(&mut self, req: AddTargetActionRequest, tx: Sender<DsResponse>) {
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

        let mut updated_target = typed_targets.get(&name).unwrap().clone();
        for action in req.actions {
            updated_target.actions.insert(action.to_ascii_lowercase());
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

    /// Remove an action from an existing target
    async fn remove_target_action(
        &mut self,
        req: RemoveTargetActionRequest,
        tx: Sender<DsResponse>,
    ) {
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

        // create a new target and update it
        let mut updated_target = typed_targets.get(&name).unwrap().clone();
        for action in req.actions {
            let action = action.to_ascii_lowercase();
            updated_target.actions.remove(&action);
        }

        // try to persist the new target to the backend and if that succeeds, update it in memory
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
}
