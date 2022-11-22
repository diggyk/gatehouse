#![warn(missing_docs)]

//! The datastore holds all the policies, targets, and internal PIP data

use flume::Receiver;
use std::collections::{HashMap, HashSet};
use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::entity::RegisteredEntity;
use crate::group::{RegisteredGroup, RegisteredGroupMember};
use crate::msgs::{DsRequest, DsResponse};
use crate::policy::{Decide, RegisteredPolicyRule};
use crate::proto::base::CheckRequest;
use crate::StorageType;

use crate::proto::entities::{
    AddEntityRequest, Entity, GetAllEntitiesRequest, ModifyEntityRequest, RemoveEntityRequest,
};
use crate::proto::groups::{
    AddGroupRequest, GetAllGroupsRequest, ModifyGroupRequest, RemoveGroupRequest,
};
use crate::proto::policies::{
    AddPolicyRequest, GetPoliciesRequest, ModifyPolicyRequest, PolicyRule, RemovePolicyRequest,
};
use crate::proto::roles::{AddRoleRequest, GetAllRolesRequest, RemoveRoleRequest, Role};
use crate::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};
use crate::role::RegisteredRole;
use crate::storage::file::FileStorage;
use crate::storage::nil::NilStorage;
use crate::storage::Storage;
use crate::target::RegisteredTarget;

pub struct Datastore {
    rx: flume::Receiver<DsRequest>,
    backend: Box<dyn Storage + Send + Sync>,

    /// HashMap from type string to HashMap of name to registered target
    targets: HashMap<String, HashMap<String, RegisteredTarget>>,

    /// HashMap from type string to HashMap of name to registered entity
    entities: HashMap<String, HashMap<String, RegisteredEntity>>,

    /// HashMap of name to registered roles
    roles: HashMap<String, RegisteredRole>,

    /// HashMap of name to registered group
    groups: HashMap<String, RegisteredGroup>,

    /// HashMap of name to registered policy
    policies: HashMap<String, RegisteredPolicyRule>,
}

impl Datastore {
    async fn new(backend: StorageType, rx: Receiver<DsRequest>) -> Self {
        let backend: Box<dyn Storage + Send + Sync> = match backend {
            StorageType::FileSystem(path) => Box::new(FileStorage::new(&path).await),
            StorageType::Nil => Box::new(NilStorage {}),
        };

        let targets = backend
            .load_targets()
            .await
            .expect("Could not load targets from backend");

        let entities = backend
            .load_entities()
            .await
            .expect("Could not load entities from backend");

        let roles = backend
            .load_roles()
            .await
            .expect("Could not load roles from backend");

        let groups = backend
            .load_groups()
            .await
            .expect("Could not load groups from backend");

        let policies = backend
            .load_policies()
            .await
            .expect("Could not load policies from backend");

        Datastore {
            rx,
            backend,
            targets,
            entities,
            roles,
            groups,
            policies,
        }
    }

    /// How the datastore is actually created, returning only the sender channel
    pub(crate) async fn create(backend: StorageType) -> flume::Sender<DsRequest> {
        let (tx, rx) = flume::unbounded();
        let mut ds = Self::new(backend, rx).await;
        tokio::spawn(async move {
            ds.run().await;
        });

        tx
    }

    /// Our main run loop.  We listen to incoming messages from the server and respond accordingly
    async fn run(&mut self) {
        while let Ok(msg) = self.rx.recv_async().await {
            match msg {
                // TARGETS
                DsRequest::AddTarget(req, tx) => self.add_target(req, tx).await,
                DsRequest::ModifyTarget(req, tx) => self.modify_target(req, tx).await,
                DsRequest::RemoveTarget(req, tx) => self.remove_target(req, tx).await,
                DsRequest::GetTargets(req, tx) => self.get_targets(req, tx).await,
                // ENTITIES
                DsRequest::AddEntity(req, tx) => self.add_entity(req, tx).await,
                DsRequest::ModifyEntity(req, tx) => self.modify_entity(req, tx).await,
                DsRequest::RemoveEntity(req, tx) => self.remove_entity(req, tx).await,
                DsRequest::GetEntities(req, tx) => self.get_entities(req, tx).await,
                // ROLES
                DsRequest::AddRole(req, tx) => self.add_role(req, tx).await,
                DsRequest::RemoveRole(req, tx) => self.remove_role(req, tx).await,
                DsRequest::GetRoles(req, tx) => self.get_roles(req, tx).await,
                // GROUPS
                DsRequest::AddGroup(req, tx) => self.add_group(req, tx).await,
                DsRequest::ModifyGroup(req, tx) => self.modify_group(req, tx).await,
                DsRequest::RemoveGroup(req, tx) => self.remove_group(req, tx).await,
                DsRequest::GetGroups(req, tx) => self.get_groups(req, tx).await,
                // POLICIES
                DsRequest::AddPolicy(req, tx) => self.add_policy(req, tx).await,
                DsRequest::ModifyPolicy(req, tx) => self.modify_policy(req, tx).await,
                DsRequest::RemovePolicy(req, tx) => self.remove_policy(req, tx).await,
                DsRequest::GetPolicies(req, tx) => self.get_policies(req, tx).await,
                // CHECKS
                DsRequest::Check(req, tx) => self.check(req, tx).await,
            }
        }

        println!("Datastore shutdown");
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

        match self.backend.save_target(&new_target).await {
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

        match self.backend.save_target(&updated_target).await {
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
        match self.backend.remove_target(&existing_target).await {
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

        match self.backend.save_entity(&new_entity).await {
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

        match self.backend.save_entity(&updated_entity).await {
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
        match self.backend.remove_entity(&existing_entity).await {
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

    /// Add a role
    async fn add_role(&mut self, req: AddRoleRequest, tx: Sender<DsResponse>) {
        let role = req.name.to_ascii_lowercase();

        let new_role = RegisteredRole::new(&role);

        // if entity already exists, return an error
        if self.roles.contains_key(&role) {
            println!("Role already exists: {}", role);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Role already exists",
            )));
            return;
        }

        // try to persist the new role to the backend and if that succeeds, update it in memory
        match self.backend.save_role(&new_role).await {
            Ok(_) => {
                let _ = self.roles.insert(role.clone(), new_role.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SingleRole(new_role.into()));
    }

    /// Remove a role
    async fn remove_role(&mut self, req: RemoveRoleRequest, tx: Sender<DsResponse>) {
        let role = req.name.to_ascii_lowercase();

        if !self.roles.contains_key(&role) {
            println!("Role does not exists: {}", role);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Role not found")));
            return;
        }

        let existing_role = self.roles.get(&role).unwrap().to_owned();

        let mut updated_groups = Vec::new();
        for group_name in &existing_role.groups {
            if let Some(grp) = self.groups.get(group_name) {
                let mut cloned_grp = grp.clone();
                cloned_grp.roles.remove(&role);
                updated_groups.push(cloned_grp);
            }
        }

        // try to remove the new role to the backend and if that succeeds, update it in memory
        match self.backend.remove_role(&existing_role.name).await {
            Ok(_) => {
                let _ = self.roles.remove(&role);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // persist the updated groups
        for updated_group in updated_groups {
            if let Err(err) = self.backend.save_group(&updated_group).await {
                // TODO! -- do something with error
                eprintln!(
                    "Persistence issue! Group {} could not be saved after remove role {}: {}",
                    updated_group.name, &role, err
                );
            }

            self.groups
                .insert(updated_group.name.clone(), updated_group);
        }

        let _ = tx.send(DsResponse::SingleRole(existing_role.into()));
    }

    /// Get all roles
    async fn get_roles(&mut self, req: GetAllRolesRequest, tx: Sender<DsResponse>) {
        let mut roles: Vec<Role> = Vec::new();

        if req.name.is_none() {
            roles = self
                .roles
                .iter()
                .map(|(_, r)| r.to_owned().into())
                .collect();
        } else if let Some(role) = self.roles.get(&req.name.unwrap()) {
            roles = vec![role.to_owned().into()];
        }
        let _ = tx.send(DsResponse::MultipleRoles(roles));
    }

    /// Add group. We cross reference role membership in the registered roles but not in entities
    /// because it is perfectly legal to have members of groups that will be expressed externally
    async fn add_group(&mut self, req: AddGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        // if entity already exists, return an error
        if self.groups.contains_key(&name) {
            println!("Group already exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Role already exists",
            )));
            return;
        }

        let members: HashSet<RegisteredGroupMember> =
            req.members.iter().map(|m| m.clone().into()).collect();

        let mut roles = HashSet::new();

        // find the existing roles we can update their references to groups
        let mut found_roles = Vec::new();
        for role_req in req.roles {
            let role_req_name = role_req.to_ascii_lowercase();
            let found_role = self.roles.get(&role_req_name);
            if found_role.is_none() {
                let _ = tx.send(DsResponse::Error(Status::not_found(format!(
                    "Role {} not found",
                    &role_req
                ))));
                return;
            }

            // make a clone of the role and add a reference to this group
            let mut cloned_role = found_role.unwrap().clone();
            cloned_role.groups.insert(name.clone().clone());
            found_roles.push(cloned_role);

            // add this role to the list of roles associated with this group
            roles.insert(role_req_name);
        }

        let new_group = RegisteredGroup::new(&name, members, roles);

        // try to save the group first and then if that works, update the roles and persist them
        if let Err(err) = self.backend.save_group(&new_group).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        self.groups.insert(name.clone(), new_group.clone());

        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.backend.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles.insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(new_group.into()));
    }

    /// Modify a group
    async fn modify_group(&mut self, req: ModifyGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        if !self.groups.contains_key(&name) {
            println!("Group does not exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Group not found")));
            return;
        }

        let mut updated_group = self.groups.get(&name).unwrap().clone();

        // add new members
        for member in req.add_members {
            updated_group.members.insert(member.into());
        }

        // remove members
        for member in req.remove_members {
            updated_group.members.remove(&member.into());
        }

        // find existing roles that are being added to this group
        let mut found_roles = Vec::new();
        for role_req in req.add_roles {
            let role_req_name = role_req.to_ascii_lowercase();
            let found_role = self.roles.get(&role_req_name);
            if found_role.is_none() {
                let _ = tx.send(DsResponse::Error(Status::not_found(format!(
                    "Role {} not found",
                    &role_req
                ))));
                return;
            }

            // make a clone of the role and add a reference to this group
            let mut cloned_role = found_role.unwrap().clone();
            cloned_role.groups.insert(name.clone());
            found_roles.push(cloned_role);

            // add this role to the list of roles associated with this group
            updated_group.roles.insert(role_req_name);
        }

        // find existing roles that are being removed from this group
        for role_req in req.remove_roles {
            let role_req_name = role_req.to_ascii_lowercase();
            let found_role = self.roles.get(&role_req_name);
            if found_role.is_none() {
                let _ = tx.send(DsResponse::Error(Status::not_found(format!(
                    "Role {} not found",
                    &role_req
                ))));
                return;
            }

            // make a clone of the role and remove the reference to this group
            let mut cloned_role = found_role.unwrap().clone();
            cloned_role.groups.remove(&name.clone());
            found_roles.push(cloned_role);

            // remove this role to the list of roles associated with this group
            updated_group.roles.remove(&role_req_name);
        }

        // try to save the group first and then if that works, update the roles and persist them
        if let Err(err) = self.backend.save_group(&updated_group).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        self.groups.insert(name.clone(), updated_group.clone());

        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.backend.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles.insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(updated_group.into()));
    }

    /// Remove an existing group
    async fn remove_group(&mut self, req: RemoveGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        if !self.groups.contains_key(&name) {
            println!("Group does not exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Group not found")));
            return;
        }

        let existing_group = self.groups.get(&name).unwrap().clone();

        // find existing roles that have been granted to this group
        let mut found_roles = Vec::new();
        // find existing roles that are being removed from this group
        for role_name in existing_group.roles.iter() {
            let found_role = self.roles.get(role_name);
            if found_role.is_none() {
                // unexpected but not the end of the world
                eprintln!(
                    "When removing group {}, the role {} didn't actually exist",
                    name, role_name
                );
            }

            // make a clone of the role and remove the reference to this group
            let mut cloned_role = found_role.unwrap().clone();
            cloned_role.groups.remove(&name.clone());
            found_roles.push(cloned_role);
        }

        // try to delete the group first and then if that works, update the roles and persist them
        if let Err(err) = self.backend.remove_group(&existing_group.name).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.backend.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles.insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(existing_group.into()));
    }

    /// Get groups based on filter
    async fn get_groups(&mut self, req: GetAllGroupsRequest, tx: Sender<DsResponse>) {
        let name_filter = req.name;
        let member_filter = req.member;
        let role_filter = req.role;

        let mut found_groups = Vec::new();

        for (name, group) in self.groups.iter() {
            if let Some(ref filter) = name_filter {
                if filter != name {
                    continue;
                }
            }

            if let Some(ref filter) = member_filter {
                let filter: RegisteredGroupMember = RegisteredGroupMember::from(filter.clone());
                if !group.members.contains(&filter) {
                    continue;
                }
            }

            if let Some(ref filter) = role_filter {
                if !group.roles.contains(filter) {
                    continue;
                }
            }

            found_groups.push(group.clone().into());
        }

        let _ = tx.send(DsResponse::MultipleGroups(found_groups));
    }

    /// Add a policy if new
    async fn add_policy(&mut self, req: AddPolicyRequest, tx: Sender<DsResponse>) {
        let rule = match req.rule {
            None => {
                let _ = tx.send(DsResponse::Error(Status::invalid_argument(
                    "No rule in request",
                )));
                return;
            }
            Some(rule) => rule,
        };

        let name = rule.name.to_ascii_lowercase();

        // if policy rule already exists, return an error
        if self.roles.contains_key(&name) {
            println!("Policy rule already exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Policy rule already exists",
            )));
            return;
        }

        let new_policy: RegisteredPolicyRule = rule.clone().into();

        // try to persist the new policy to the backend and if that succeeds, update it in memory
        match self.backend.save_policy(&new_policy).await {
            Ok(_) => {
                let _ = self.policies.insert(name.clone(), new_policy.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(new_policy.into()));
    }

    /// Update an existing policy
    async fn modify_policy(&mut self, req: ModifyPolicyRequest, tx: Sender<DsResponse>) {
        let rule = match req.rule {
            None => {
                let _ = tx.send(DsResponse::Error(Status::invalid_argument(
                    "No rule in request",
                )));
                return;
            }
            Some(rule) => rule,
        };

        let name = rule.name.to_ascii_lowercase();

        // if policy rule does not exist, return an error
        if !self.policies.contains_key(&name) {
            println!("Policy rule not found: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Policy rule does not exist",
            )));
            return;
        }

        let updated_policy: RegisteredPolicyRule = rule.clone().into();

        // try to persist the new policy to the backend and if that succeeds, update it in memory
        match self.backend.save_policy(&updated_policy).await {
            Ok(_) => {
                let _ = self.policies.insert(name.clone(), updated_policy.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(updated_policy.into()));
    }

    /// Remove an existing policy
    async fn remove_policy(&mut self, req: RemovePolicyRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        // if policy rule does not exist, return an error
        if !self.policies.contains_key(&name) {
            println!("Policy rule not found: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Policy rule does not exist",
            )));
            return;
        }

        let existing_policy = self.policies.get(&name).unwrap().to_owned();

        // try to remove the policy from backend before updating memory
        match self.backend.remove_policy(&name).await {
            Ok(_) => {
                let _ = self.policies.remove(&name);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(existing_policy.into()));
    }

    /// Get policies based on filters
    async fn get_policies(&mut self, req: GetPoliciesRequest, tx: Sender<DsResponse>) {
        let mut policies: Vec<PolicyRule> = Vec::new();

        let req_name = req.name.map(|n| n.to_ascii_lowercase());
        for (name, policy) in self.policies.iter() {
            // see if name matches if a name filter was given
            if let Some(ref req_name) = req_name {
                if name != req_name {
                    continue;
                }
            }

            // see if entity check matches if entity check filter was given
            if req.entity_check.is_some() {
                // NOT IMPLEMENTED
                let _ = tx.send(DsResponse::Error(Status::unimplemented(
                    "Filter by entity check not implemented",
                )));
                return;
            }

            // see if env attributes match
            if !req.env_attributes.is_empty() {
                // NOT IMPLEMENTED
                let _ = tx.send(DsResponse::Error(Status::unimplemented(
                    "Filter by environment attribute checks is not implemented",
                )));
                return;
            }

            // see if target check matches if target check filter was given
            if req.target_check.is_some() {
                // NOT IMPLEMENTED
                let _ = tx.send(DsResponse::Error(Status::unimplemented(
                    "Filter by target check not implemented",
                )));
                return;
            }

            policies.push(policy.to_owned().into());
        }

        let _ = tx.send(DsResponse::MultiplePolicies(policies));
    }

    /// Perform a check
    ///
    /// We will receive an entity (type and name) and a list of attributes that the policy
    /// enforcement point (PEP) wants to share about the entity. PEP may also share environment
    /// attributes in the form of key/val pairs (vals are a list). Finally, the PEP will specify
    /// the target (name and type) and action to be checked against the policies. DS will evaluate
    /// against known policy rules and decide on a PASS/FAIL decision.
    ///
    /// The entity may match a registered entity, in which case, we'll add some more attributes if
    /// we have them. The entity may also belong to a group which has been granted some roles.
    /// In that case, we will add "member-of" attributes for each group, and "has-role" attributes
    /// for each role. Lastly, we will determine a bucket (between 0-99) using the murmur3 algo
    async fn check(&mut self, req: CheckRequest, tx: Sender<DsResponse>) {
        let entity: RegisteredEntity = self.extend_entity(req.entity.unwrap());
        let mut env_attributes = HashMap::new();
        for (key, vals) in req.env_attributes {
            env_attributes.insert(key, HashSet::from_iter(vals.values));
        }

        // get any known attributes about the target
        let target_attributes = self.get_target_attributes(&req.target_name, &req.target_type);

        // TODO -- refactor the policy store to make applicable polices quicker to find
        // Examine every policy -- if the entity check, environment check, and target check's pass
        // then we can make a determination. If we get an explicit DENY from any rule, we exit
        // immediately.
        let mut decision = Decide::Fail;
        for (_, policy) in self.policies.iter() {
            if let Some(ref entity_check) = policy.entity_check {
                if !entity_check.check(&entity) {
                    // this entity check does not apply to this request
                    continue;
                }
            }

            // perform environment check
            if !policy
                .env_attributes
                .iter()
                .all(|ea| ea.check(&env_attributes))
            {
                // these environment checks do not match
                continue;
            }

            if let Some(ref target_check) = policy.target_check {
                if !target_check.check(
                    &req.target_name,
                    &req.target_type,
                    &target_attributes,
                    &req.target_action,
                ) {
                    // this target does not match
                    continue;
                }
            }

            // all conditions must match; take decision
            decision = policy.decision.clone();
            if let Decide::Fail = decision {
                break;
            }
        }

        let _ = tx.send(DsResponse::CheckResult(decision.into()));
    }

    /** HELPERS */
    /// Extend a given entity with additional attributes
    ///
    /// Given an entity, build a registered entity with any additional attributes from a
    /// known entity, as well as any group/roles we might have.
    fn extend_entity(&self, passed_entity: Entity) -> RegisteredEntity {
        let mut entity: RegisteredEntity = RegisteredEntity::from(passed_entity);
        let typed_entities = self.entities.get(&entity.typestr);

        // extend attributes if we know about this entity
        if let Some(typed_entities) = typed_entities {
            if let Some(found_entity) = typed_entities.get(&entity.name).map(|e| e.to_owned()) {
                entity
                    .attributes
                    .extend(found_entity.attributes.into_iter());
            }
        }

        // check groups
        // create a representation of this entity as a RegisteredGroupMember so we can
        let entity_as_member = RegisteredGroupMember::from(&entity);
        for group in self.groups.values() {
            if group.members.contains(&entity_as_member) {
                entity
                    .attributes
                    .entry("member-of".to_string())
                    .or_default()
                    .insert(group.name.clone());

                entity
                    .attributes
                    .entry("has-role".to_string())
                    .or_default()
                    .extend(group.roles.clone());
            }
        }

        entity
    }

    /// Return attributes for a target if known
    fn get_target_attributes(&self, name: &str, typestr: &str) -> HashMap<String, HashSet<String>> {
        let typed_targets = self.targets.get(typestr);

        if let Some(typed_targets) = typed_targets {
            if let Some(found_target) = typed_targets.get(name) {
                return found_target.attributes.clone();
            }
        }

        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot::channel;
    use tokio::test;

    use crate::proto::common::AttributeValues;

    use super::*;

    fn str(val: &str) -> String {
        String::from(val)
    }

    #[test]
    async fn test_targets() {
        let (_, rx) = flume::unbounded();
        let (tx, _) = channel::<DsResponse>();
        let mut ds = Datastore::new(StorageType::Nil, rx).await;

        let mut map: HashMap<String, AttributeValues> = HashMap::new();
        map.insert(
            str("role"),
            AttributeValues {
                values: vec![str("main"), str("backup")],
            },
        );
        map.insert(
            str("env"),
            AttributeValues {
                values: vec![str("test")],
            },
        );

        let req = AddTargetRequest {
            name: str("test"),
            typestr: str("typetest"),
            actions: vec![str("action1"), str("action2")],
            attributes: map,
        };
        ds.add_target(req, tx).await;

        assert_eq!(ds.targets.len(), 1);
        assert!(ds.targets.contains_key("typetest"));
        assert!(ds.targets.get("typetest").unwrap().contains_key("test"));
    }

    // TODO! -- add more unit tests
}
