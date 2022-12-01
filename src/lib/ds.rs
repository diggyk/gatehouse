#![warn(missing_docs)]

//! The datastore holds all the policies, targets, and internal PIP data

use flume::Receiver;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::RwLock;
use tonic::Status;

use crate::actor::RegisteredActor;
use crate::group::{RegisteredGroup, RegisteredGroupMember};
use crate::msgs::{DsRequest, DsResponse};
use crate::policy::{Decide, RegisteredPolicyRule};
use crate::proto::base::CheckRequest;
use crate::StorageType;

use crate::proto::actors::{
    Actor, AddActorRequest, GetActorsRequest, ModifyActorRequest, RemoveActorRequest,
};
use crate::proto::groups::{
    AddGroupRequest, GetGroupsRequest, ModifyGroupRequest, RemoveGroupRequest,
};
use crate::proto::policies::{
    AddPolicyRequest, GetPoliciesRequest, ModifyPolicyRequest, PolicyRule, RemovePolicyRequest,
};
use crate::proto::roles::{AddRoleRequest, GetRolesRequest, RemoveRoleRequest, Role};
use crate::proto::targets::{
    AddTargetRequest, GetTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};
use crate::role::RegisteredRole;
use crate::storage::etcd::EtcdStorage;
use crate::storage::file::FileStorage;
use crate::storage::nil::NilStorage;
use crate::storage::{BackendUpdate, Storage};
use crate::target::RegisteredTarget;

pub struct Datastore {
    rx: flume::Receiver<DsRequest>,
    storage: Box<dyn Storage + Send + Sync>,

    /// HashMap from type string to HashMap of name to registered target
    targets: Arc<RwLock<HashMap<String, HashMap<String, RegisteredTarget>>>>,

    /// HashMap from type string to HashMap of name to registered actor
    actors: Arc<RwLock<HashMap<String, HashMap<String, RegisteredActor>>>>,

    /// HashMap of name to registered roles
    roles: Arc<RwLock<HashMap<String, RegisteredRole>>>,

    /// HashMap of name to registered group
    groups: Arc<RwLock<HashMap<String, RegisteredGroup>>>,

    /// HashMap of name to registered policy
    policies: Arc<RwLock<HashMap<String, RegisteredPolicyRule>>>,
}

impl Datastore {
    async fn new(
        backend: &StorageType,
        req_tx: flume::Sender<DsRequest>,
        req_rx: Receiver<DsRequest>,
    ) -> Self {
        let backend: Box<dyn Storage + Send + Sync> = match backend {
            StorageType::Etcd(url) => Box::new(EtcdStorage::new(url, req_tx).await),
            StorageType::FileSystem(path) => Box::new(FileStorage::new(path).await),
            StorageType::Nil => Box::new(NilStorage {}),
        };

        let targets = backend
            .load_targets()
            .await
            .expect("Could not load targets from backend");

        let actors = backend
            .load_actors()
            .await
            .expect("Could not load actors from backend");

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
            rx: req_rx,
            storage: backend,
            targets: Arc::new(RwLock::new(targets)),
            actors: Arc::new(RwLock::new(actors)),
            roles: Arc::new(RwLock::new(roles)),
            groups: Arc::new(RwLock::new(groups)),
            policies: Arc::new(RwLock::new(policies)),
        }
    }

    /// How the datastore is actually created, returning only the sender channel
    pub(crate) async fn create(backend: &StorageType) -> flume::Sender<DsRequest> {
        let (req_tx, req_rx) = flume::unbounded();
        let ds = Self::new(backend, req_tx.clone(), req_rx).await;

        let arc_ds = Arc::new(ds);
        tokio::spawn(async move {
            arc_ds.run().await;
        });

        req_tx
    }

    /// Our main run loop.  We listen to incoming messages from the server and respond accordingly
    async fn run(self: Arc<Self>) {
        while let Ok(msg) = self.rx.recv_async().await {
            let me = Arc::clone(&self);
            match msg {
                // TARGETS
                DsRequest::AddTarget(req, tx) => {
                    tokio::spawn(async move { me.add_target(req, tx).await });
                }
                DsRequest::ModifyTarget(req, tx) => {
                    tokio::spawn(async move { me.modify_target(req, tx).await });
                }
                DsRequest::RemoveTarget(req, tx) => {
                    tokio::spawn(async move { me.remove_target(req, tx).await });
                }
                DsRequest::GetTargets(req, tx) => {
                    tokio::spawn(async move { me.get_targets(req, tx).await });
                }
                // ENTITIES
                DsRequest::AddActor(req, tx) => {
                    tokio::spawn(async move { me.add_actor(req, tx).await });
                }
                DsRequest::ModifyActor(req, tx) => {
                    tokio::spawn(async move { me.modify_actor(req, tx).await });
                }
                DsRequest::RemoveActor(req, tx) => {
                    tokio::spawn(async move { me.remove_actor(req, tx).await });
                }
                DsRequest::GetActors(req, tx) => {
                    tokio::spawn(async move { me.get_actors(req, tx).await });
                }
                // ROLES
                DsRequest::AddRole(req, tx) => {
                    tokio::spawn(async move { me.add_role(req, tx).await });
                }
                DsRequest::RemoveRole(req, tx) => {
                    tokio::spawn(async move { me.remove_role(req, tx).await });
                }
                DsRequest::GetRoles(req, tx) => {
                    tokio::spawn(async move { me.get_roles(req, tx).await });
                }
                // GROUPS
                DsRequest::AddGroup(req, tx) => {
                    tokio::spawn(async move { me.add_group(req, tx).await });
                }
                DsRequest::ModifyGroup(req, tx) => {
                    tokio::spawn(async move { me.modify_group(req, tx).await });
                }
                DsRequest::RemoveGroup(req, tx) => {
                    tokio::spawn(async move { me.remove_group(req, tx).await });
                }
                DsRequest::GetGroups(req, tx) => {
                    tokio::spawn(async move { me.get_groups(req, tx).await });
                }
                // POLICIES
                DsRequest::AddPolicy(req, tx) => {
                    tokio::spawn(async move { me.add_policy(req, tx).await });
                }
                DsRequest::ModifyPolicy(req, tx) => {
                    tokio::spawn(async move { me.modify_policy(req, tx).await });
                }
                DsRequest::RemovePolicy(req, tx) => {
                    tokio::spawn(async move { me.remove_policy(req, tx).await });
                }
                DsRequest::GetPolicies(req, tx) => {
                    tokio::spawn(async move { me.get_policies(req, tx).await });
                }
                // CHECKS
                DsRequest::Check(req, tx) => {
                    tokio::spawn(async move { me.check(req, tx).await });
                }
                // UPDATES FROM BACKEND
                DsRequest::Update(req) => {
                    tokio::spawn(async move { me.update(req).await });
                }
            }
        }

        println!("Datastore shutdown");
    }

    /// Add a new target
    async fn add_target(&self, req: AddTargetRequest, tx: Sender<DsResponse>) {
        // add to the local cache
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // get or create the hashmap for this "type" of target
        let mut targets = self.targets.write().await;
        let typed_targets = targets.entry(typestr.clone()).or_insert_with(HashMap::new);

        // if target already exists, return an error
        if typed_targets.contains_key(&name) {
            println!("Target already exists: {}/{}", typestr, name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Target already exists",
            )));
            return;
        }
        drop(targets);

        // convert the attributes to a hashmap
        let mut attributes = HashMap::new();
        for attrib in req.attributes {
            attributes.insert(attrib.0, HashSet::from_iter(attrib.1.values));
        }

        let new_target = RegisteredTarget::new(&name, &typestr, req.actions, attributes);

        match self.storage.save_target(&new_target).await {
            Ok(_) => {
                let mut targets = self.targets.write().await;
                let typed_targets = targets.get_mut(&typestr).unwrap();
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
    async fn modify_target(&self, req: ModifyTargetRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        let targets = self.targets.read().await;

        if !targets.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by type",
            )));
            return;
        }

        let typed_targets = targets.get(&typestr).unwrap();
        if !(typed_targets.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by name",
            )));
            return;
        }

        // we'll work with a clone of the target in case persistence fails
        let mut updated_target = typed_targets.get(&name).unwrap().clone();

        // release the lock
        drop(targets);

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

        match self.storage.save_target(&updated_target).await {
            Ok(_) => {
                let mut targets = self.targets.write().await;
                let typed_targets = targets.get_mut(&typestr).unwrap();
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
    async fn remove_target(&self, req: RemoveTargetRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // make sure the target type exists
        if !self.targets.read().await.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by type",
            )));
            return;
        }

        // make sure the target exists
        let targets = self.targets.read().await;
        let typed_targets = targets.get(&typestr).unwrap();
        if !(typed_targets.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find target by name",
            )));
            return;
        }

        // get the existing target
        let existing_target = typed_targets.get(&name).unwrap().clone();

        // explicitly drop targets to release lock
        drop(targets);

        // try to persist the new target to the backend and if that succeeds, update it in memory
        match self.storage.remove_target(&existing_target).await {
            Ok(_) => {
                let mut targets = self.targets.write().await;
                let typed_targets = targets.get_mut(&typestr).unwrap();
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
    async fn get_targets(&self, req: GetTargetsRequest, tx: Sender<DsResponse>) {
        let typestr = req.typestr.map(|t| t.to_ascii_lowercase());
        let name = req.name.map(|t| t.to_ascii_lowercase());
        let mut found_targets: Vec<Target> = Vec::new();

        for typemap in self.targets.read().await.iter() {
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
                found_targets.push(target.1.clone().into());
            }
        }

        let _ = tx.send(DsResponse::MultipleTargets(found_targets));
    }

    /// Add a new actor
    async fn add_actor(&self, req: AddActorRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        // get or create the hashmap for this "type" of target
        let mut actors = self.actors.write().await;
        let typed_actors = actors.entry(typestr.clone()).or_insert_with(HashMap::new);

        // if actor already exists, return an error
        if typed_actors.contains_key(&name) {
            println!("Actor already exists: {}/{}", typestr, name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Actor already exists",
            )));
            return;
        }

        // drop the lock
        drop(actors);

        // convert the attributes to a hashmap
        let mut attributes = HashMap::new();
        for (key, vals) in req.attributes {
            attributes.insert(key, HashSet::from_iter(vals.values));
        }

        let new_actor = RegisteredActor::new(&name, &typestr, attributes);

        match self.storage.save_actor(&new_actor).await {
            Ok(_) => {
                let mut actors = self.actors.write().await;
                let typed_actors = actors.get_mut(&typestr).unwrap();
                typed_actors.insert(name, new_actor.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleActor(new_actor.into()));
    }

    /// Modify and existing actor
    async fn modify_actor(&self, req: ModifyActorRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        let actors = self.actors.read().await;

        if !actors.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find actor by type",
            )));
            return;
        }

        let typed_actors = actors.get(&typestr).unwrap();
        if !(typed_actors.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find actor by name",
            )));
            return;
        }

        // we'll work with a clone of the actor in case persistence fails
        let mut updated_actor = typed_actors.get(&name).unwrap().clone();

        // drop the lock
        drop(actors);

        // update attributes
        for attrib in req.add_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            let attrib_entry = updated_actor.attributes.entry(key).or_default();
            attrib_entry.extend(values);
        }
        for attrib in req.remove_attributes {
            let key = attrib.0;
            let values = attrib.1.values;

            if let Some(current_values) = updated_actor.attributes.get_mut(&key) {
                for value in values {
                    current_values.remove(&value);
                }

                if current_values.is_empty() {
                    updated_actor.attributes.remove(&key);
                }
            }
        }

        match self.storage.save_actor(&updated_actor).await {
            Ok(_) => {
                let mut actors = self.actors.write().await;
                let typed_actors = actors.get_mut(&typestr).unwrap();
                let _ = typed_actors.insert(name.clone(), updated_actor.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleActor(updated_actor.clone().into()));
    }

    /// Remove an existing actor
    async fn remove_actor(&self, req: RemoveActorRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();
        let typestr = req.typestr.to_ascii_lowercase();

        let actors = self.actors.read().await;

        // make sure the target type exists
        if !actors.contains_key(&typestr) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find actor by type",
            )));
            return;
        }

        // make sure the target exists
        let typed_actors = actors.get(&typestr).unwrap();
        if !(typed_actors.contains_key(&name)) {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Could not find actor by name",
            )));
            return;
        }

        // get the existing actor
        let existing_actor = typed_actors.get(&name).unwrap().clone();

        // drop the lock
        drop(actors);

        // try to persist the new target to the backend and if that succeeds, update it in memory
        match self.storage.remove_actor(&existing_actor).await {
            Ok(_) => {
                let mut actors = self.actors.write().await;
                let typed_actors = actors.get_mut(&typestr).unwrap();
                let _ = typed_actors.remove(&name);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // TODO! -- do something with error
        let _ = tx.send(DsResponse::SingleActor(existing_actor.clone().into()));
    }

    /// Get all actors, optionally filtered by type
    async fn get_actors(&self, req: GetActorsRequest, tx: Sender<DsResponse>) {
        let type_filter = req.typestr.map(|t| t.to_ascii_lowercase());
        let name_filter = req.name.map(|t| t.to_ascii_lowercase());
        let mut found_actors: Vec<Actor> = Vec::new();

        let actors = self.actors.read().await;

        for (typestr, actors_of_type) in actors.iter() {
            if let Some(ref filter_type) = type_filter {
                if typestr.as_str() != filter_type {
                    continue;
                }
            }
            for (actor_name, actor) in actors_of_type.iter() {
                if let Some(ref name_ref) = name_filter {
                    if actor_name.as_str() != name_ref {
                        continue;
                    }
                }
                let expanded_actor = self.expand_groups_and_roles(actor.clone()).await;
                found_actors.push(expanded_actor.into());
            }
        }

        let _ = tx.send(DsResponse::MultipleActors(found_actors));
    }

    /// Add a role
    async fn add_role(&self, req: AddRoleRequest, tx: Sender<DsResponse>) {
        let role = req.name.to_ascii_lowercase();

        let new_role = RegisteredRole::new(&role);

        // if actor already exists, return an error
        if self.roles.read().await.contains_key(&role) {
            println!("Role already exists: {}", role);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Role already exists",
            )));
            return;
        }

        // try to persist the new role to the backend and if that succeeds, update it in memory
        match self.storage.save_role(&new_role).await {
            Ok(_) => {
                let _ = self
                    .roles
                    .write()
                    .await
                    .insert(role.clone(), new_role.clone());
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
    async fn remove_role(&self, req: RemoveRoleRequest, tx: Sender<DsResponse>) {
        let role = req.name.to_ascii_lowercase();

        if !self.roles.read().await.contains_key(&role) {
            println!("Role does not exists: {}", role);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Role not found")));
            return;
        }

        let existing_role = self.roles.read().await.get(&role).unwrap().to_owned();

        let mut updated_groups = Vec::new();
        for group_name in &existing_role.groups {
            if let Some(grp) = self.groups.read().await.get(group_name) {
                let mut cloned_grp = grp.clone();
                cloned_grp.roles.remove(&role);
                updated_groups.push(cloned_grp);
            }
        }

        // try to remove the new role from the backend and if that succeeds, update it in memory
        match self.storage.remove_role(&existing_role.name).await {
            Ok(_) => {
                let _ = self.roles.write().await.remove(&role);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        // persist the updated groups
        for updated_group in updated_groups {
            if let Err(err) = self.storage.save_group(&updated_group).await {
                // TODO! -- do something with error
                eprintln!(
                    "Persistence issue! Group {} could not be saved after removing role {}: {}",
                    updated_group.name, &role, err
                );
            }

            self.groups
                .write()
                .await
                .insert(updated_group.name.clone(), updated_group);
        }

        let _ = tx.send(DsResponse::SingleRole(existing_role.into()));
    }

    /// Get all roles
    async fn get_roles(&self, req: GetRolesRequest, tx: Sender<DsResponse>) {
        let mut roles: Vec<Role> = Vec::new();

        if req.name.is_none() {
            roles = self
                .roles
                .read()
                .await
                .iter()
                .map(|(_, r)| r.to_owned().into())
                .collect();
        } else if let Some(role) = self.roles.read().await.get(&req.name.unwrap()) {
            roles = vec![role.to_owned().into()];
        }
        let _ = tx.send(DsResponse::MultipleRoles(roles));
    }

    /// Add group. We cross reference role membership in the registered roles but not in actors
    /// because it is perfectly legal to have members of groups that will be expressed externally
    async fn add_group(&self, req: AddGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        // if actor already exists, return an error
        if self.groups.read().await.contains_key(&name) {
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
            let known_roles = self.roles.read().await;
            let found_role = known_roles.get(&role_req_name);
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

        let new_group = RegisteredGroup::new(&name, req.desc, members, roles);

        // try to save the group first and then if that works, update the roles and persist them
        if let Err(err) = self.storage.save_group(&new_group).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        self.groups
            .write()
            .await
            .insert(name.clone(), new_group.clone());

        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.storage.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles
                .write()
                .await
                .insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(new_group.into()));
    }

    /// Modify a group
    async fn modify_group(&self, req: ModifyGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        if !self.groups.read().await.contains_key(&name) {
            println!("Group does not exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Group not found")));
            return;
        }

        let mut updated_group = self.groups.read().await.get(&name).unwrap().clone();

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
            let known_roles = self.roles.read().await;
            let found_role = known_roles.get(&role_req_name);
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
            let known_roles = self.roles.read().await;
            let found_role = known_roles.get(&role_req_name);
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
        if let Err(err) = self.storage.save_group(&updated_group).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        self.groups
            .write()
            .await
            .insert(name.clone(), updated_group.clone());

        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.storage.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles
                .write()
                .await
                .insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(updated_group.into()));
    }

    /// Remove an existing group
    async fn remove_group(&self, req: RemoveGroupRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        if !self.groups.read().await.contains_key(&name) {
            println!("Group does not exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found("Group not found")));
            return;
        }

        let existing_group = self.groups.read().await.get(&name).unwrap().clone();

        // find existing roles that have been granted to this group
        let mut found_roles = Vec::new();
        // find existing roles that are being removed from this group
        for role_name in existing_group.roles.iter() {
            let known_roles = self.roles.read().await;
            let found_role = known_roles.get(role_name);
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
        if let Err(err) = self.storage.remove_group(&existing_group.name).await {
            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::internal(err)));
            return;
        }

        // update the roles so they are no longer pointing to this deleted group
        for found_role in found_roles {
            // if this fails, we have a referential integrity problem
            if let Err(err) = self.storage.save_role(&found_role).await {
                // TODO! -- really alert on this error
                eprintln!("Referential integrity issue: role {} could not be saved after adding to group {}: {}", found_role.name, &name, err);
            }
            self.roles
                .write()
                .await
                .insert(found_role.name.clone(), found_role);
        }

        let _ = tx.send(DsResponse::SingleGroup(existing_group.into()));
    }

    /// Get groups based on filter
    async fn get_groups(&self, req: GetGroupsRequest, tx: Sender<DsResponse>) {
        let name_filter = req.name;
        let member_filter = req.member;
        let role_filter = req.role;

        let mut found_groups = Vec::new();

        for (name, group) in self.groups.read().await.iter() {
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
    async fn add_policy(&self, req: AddPolicyRequest, tx: Sender<DsResponse>) {
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
        if self.policies.read().await.contains_key(&name) {
            println!("Policy rule already exists: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::already_exists(
                "Policy rule already exists",
            )));
            return;
        }

        let new_policy: RegisteredPolicyRule = rule.clone().into();

        // try to persist the new policy to the backend and if that succeeds, update it in memory
        match self.storage.save_policy(&new_policy).await {
            Ok(_) => {
                let _ = self
                    .policies
                    .write()
                    .await
                    .insert(name.clone(), new_policy.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(Box::new(new_policy.into())));
    }

    /// Update an existing policy
    async fn modify_policy(&self, req: ModifyPolicyRequest, tx: Sender<DsResponse>) {
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
        if !self.policies.read().await.contains_key(&name) {
            println!("Policy rule not found: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Policy rule does not exist",
            )));
            return;
        }

        let updated_policy: RegisteredPolicyRule = rule.clone().into();

        // try to persist the new policy to the backend and if that succeeds, update it in memory
        match self.storage.save_policy(&updated_policy).await {
            Ok(_) => {
                let _ = self
                    .policies
                    .write()
                    .await
                    .insert(name.clone(), updated_policy.clone());
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(Box::new(updated_policy.into())));
    }

    /// Remove an existing policy
    async fn remove_policy(&self, req: RemovePolicyRequest, tx: Sender<DsResponse>) {
        let name = req.name.to_ascii_lowercase();

        // if policy rule does not exist, return an error
        if !self.policies.read().await.contains_key(&name) {
            println!("Policy rule not found: {}", name);

            // TODO! -- do something with error
            let _ = tx.send(DsResponse::Error(Status::not_found(
                "Policy rule does not exist",
            )));
            return;
        }

        let existing_policy = self.policies.read().await.get(&name).unwrap().to_owned();

        // try to remove the policy from backend before updating memory
        match self.storage.remove_policy(&name).await {
            Ok(_) => {
                let _ = self.policies.write().await.remove(&name);
            }
            Err(err) => {
                // TODO! -- do something with error
                let _ = tx.send(DsResponse::Error(Status::internal(err)));
                return;
            }
        }

        let _ = tx.send(DsResponse::SinglePolicy(Box::new(existing_policy.into())));
    }

    /// Get policies based on filters
    async fn get_policies(&self, req: GetPoliciesRequest, tx: Sender<DsResponse>) {
        let mut policies: Vec<PolicyRule> = Vec::new();

        let req_name = req.name.map(|n| n.to_ascii_lowercase());
        for (name, policy) in self.policies.read().await.iter() {
            // see if name matches if a name filter was given
            if let Some(ref req_name) = req_name {
                if name != req_name {
                    continue;
                }
            }

            policies.push(policy.to_owned().into());
        }

        let _ = tx.send(DsResponse::MultiplePolicies(policies));
    }

    /// Update data directly
    ///
    /// We get these updates from the backend when working in a distributed model so we
    /// want to update the data in the memory store when getting these updates
    /// TODO -- we should only update the memory when we get this call from the backend
    ///         rather than doing it directly on an update request. And we should do it
    ///         in transactions when changing multiple entities at once, like when adding
    ///         roles to groups
    async fn update(&self, req: BackendUpdate) {
        match req {
            BackendUpdate::PutActor(actor) => {
                println!("backend => add actor {}/{}", actor.typestr, actor.name);
                let mut actors = self.actors.write().await;
                let typed_actors = actors
                    .entry(actor.typestr.clone())
                    .or_insert_with(HashMap::new);
                typed_actors.insert(actor.name.clone(), actor);
            }
            BackendUpdate::PutGroup(group) => {
                println!("backend => add group {}", group.name);
                let mut groups = self.groups.write().await;
                groups.insert(group.name.clone(), group);
            }
            BackendUpdate::PutPolicyRule(policyrule) => {
                println!("backend => add policy {}", policyrule.name);
                let mut policies = self.policies.write().await;
                policies.insert(policyrule.name.clone(), policyrule);
            }
            BackendUpdate::PutRole(role) => {
                println!("backend => add role {}", role.name);
                let mut roles = self.roles.write().await;
                roles.insert(role.name.clone(), role);
            }
            BackendUpdate::PutTarget(target) => {
                println!("backend => add target {}", target.name);
                let mut targets = self.targets.write().await;
                let type_targets = targets
                    .entry(target.typestr.clone())
                    .or_insert_with(HashMap::new);
                type_targets.insert(target.name.clone(), target);
            }
            BackendUpdate::DeleteActor(typestr, name) => {
                println!("backend => delete {}/{}", typestr, name);
                let mut actors = self.actors.write().await;
                if let Some(typed_actors) = actors.get_mut(&typestr) {
                    typed_actors.remove(&name);
                }
            }
            BackendUpdate::DeleteGroup(name) => {
                println!("backend => delete group {}", name);
                let mut groups = self.groups.write().await;
                groups.remove(&name);
            }
            BackendUpdate::DeletePolicyRule(name) => {
                println!("backend => delete policy rule {}", name);
                let mut policies = self.policies.write().await;
                policies.remove(&name);
            }
            BackendUpdate::DeleteRole(name) => {
                println!("backend => delete role {}", name);
                let mut roles = self.roles.write().await;
                roles.remove(&name);
            }
            BackendUpdate::DeleteTarget(typestr, name) => {
                println!("backend => delete target {}/{}", typestr, name);
                let mut targets = self.targets.write().await;
                if let Some(typed_targets) = targets.get_mut(&typestr) {
                    typed_targets.remove(&name);
                }
            }
        }
    }

    /// Perform a check
    ///
    /// We will receive an actor (type and name) and a list of attributes that the policy
    /// enforcement point (PEP) wants to share about the actor. PEP may also share environment
    /// attributes in the form of key/val pairs (vals are a list). Finally, the PEP will specify
    /// the target (name and type) and action to be checked against the policies. DS will evaluate
    /// against known policy rules and decide on a ALLOW/DENY decision.
    ///
    /// The actor may match a registered actor, in which case, we'll add some more attributes if
    /// we have them. The actor may also belong to a group which has been granted some roles.
    /// In that case, we will add "member-of" attributes for each group, and "has-role" attributes
    /// for each role. Lastly, we will determine a bucket (between 0-99) using the murmur3 algo
    async fn check(&self, req: CheckRequest, tx: Sender<DsResponse>) {
        let actor = self
            .extend_actor(RegisteredActor::from(req.actor.unwrap()))
            .await;

        let mut env_attributes = HashMap::new();
        for (key, vals) in req.env_attributes {
            env_attributes.insert(key, HashSet::from_iter(vals.values));
        }

        // get any known attributes about the target
        let target_attributes = self
            .get_target_attributes(&req.target_name, &req.target_type)
            .await;

        // TODO -- refactor the policy store to make applicable polices quicker to find
        // Examine every policy -- if the actor check, environment check, and target check's pass
        // then we can make a determination. If we get an explicit DENY from any rule, we exit
        // immediately.
        let mut decision = Decide::Deny;
        for (_, policy) in self.policies.read().await.iter() {
            if let Some(ref actor_check) = policy.actor_check {
                if !actor_check.check(&actor) {
                    // this actor check does not apply to this request
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
                    &actor.attributes,
                    &env_attributes,
                ) {
                    // this target does not match
                    continue;
                }
            }

            // all conditions must match; take decision
            decision = policy.decision.clone();
            if let Decide::Deny = decision {
                break;
            }
        }

        let _ = tx.send(DsResponse::CheckResult(decision.into()));
    }

    /** HELPERS */
    /// Extend a given actor with additional attributes
    ///
    /// Given a RegisteredActor created just from a gRPC call,
    /// update it with any additional attributes from a
    /// known actor, as well as any group/roles we might have.
    async fn extend_actor(&self, mut actor: RegisteredActor) -> RegisteredActor {
        let actors = self.actors.read().await;
        let typed_actors = actors.get(&actor.typestr);

        // extend attributes if we know about this actor
        if let Some(typed_actors) = typed_actors {
            if let Some(found_actor) = typed_actors.get(&actor.name).map(|e| e.to_owned()) {
                actor.attributes.extend(found_actor.attributes.into_iter());
            }
        }

        actor = self.expand_groups_and_roles(actor).await;

        actor
    }

    async fn expand_groups_and_roles(&self, mut actor: RegisteredActor) -> RegisteredActor {
        // create a representation of this actor as a RegisteredGroupMember so we can search for it
        let actor_as_member = RegisteredGroupMember::from(&actor);

        let groups = self.groups.read().await;

        for group in groups.values() {
            if group.members.contains(&actor_as_member) {
                actor
                    .attributes
                    .entry("member-of".to_string())
                    .or_default()
                    .insert(group.name.clone());

                actor
                    .attributes
                    .entry("has-role".to_string())
                    .or_default()
                    .extend(group.roles.clone());
            }
        }

        actor
    }

    /// Return attributes for a target if known
    async fn get_target_attributes(
        &self,
        name: &str,
        typestr: &str,
    ) -> HashMap<String, HashSet<String>> {
        let targets = self.targets.read().await;
        let typed_targets = targets.get(typestr);

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
        let (req_tx, req_rx) = flume::unbounded();
        let (tx, _) = channel::<DsResponse>();
        let ds = Datastore::new(&StorageType::Nil, req_tx, req_rx).await;

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

        assert_eq!(ds.targets.read().await.len(), 1);
        assert!(ds.targets.read().await.contains_key("typetest"));
        assert!(ds
            .targets
            .read()
            .await
            .get("typetest")
            .unwrap()
            .contains_key("test"));
    }

    // TODO! -- add more unit tests
}
