use std::collections::HashMap;

use tonic::async_trait;

use crate::actor::RegisteredActor;
use crate::group::RegisteredGroup;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::target::RegisteredTarget;

pub(crate) mod etcd;
pub(crate) mod file;
pub(crate) mod nil;

#[derive(Debug)]
pub(crate) enum BackendUpdate {
    PutActor(RegisteredActor),
    PutGroup(RegisteredGroup),
    PutPolicyRule(RegisteredPolicyRule),
    PutRole(RegisteredRole),
    PutTarget(RegisteredTarget),
    DeleteActor(String, String),
    DeleteGroup(String),
    DeletePolicyRule(String),
    DeleteRole(String),
    DeleteTarget(String, String),
}

#[async_trait]
pub(crate) trait Storage {
    async fn save_target(&self, tgt: &RegisteredTarget) -> Result<(), String>;
    async fn remove_target(&self, tgt: &RegisteredTarget) -> Result<(), String>;
    async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String>;
    async fn save_actor(&self, tgt: &RegisteredActor) -> Result<(), String>;
    async fn remove_actor(&self, tgt: &RegisteredActor) -> Result<(), String>;
    async fn load_actors(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredActor>>, String>;
    async fn save_role(&self, role: &RegisteredRole) -> Result<(), String>;
    async fn remove_role(&self, name: &str) -> Result<(), String>;
    async fn load_roles(&self) -> Result<HashMap<String, RegisteredRole>, String>;
    async fn save_group(&self, group: &RegisteredGroup) -> Result<(), String>;
    async fn remove_group(&self, name: &str) -> Result<(), String>;
    async fn load_groups(&self) -> Result<HashMap<String, RegisteredGroup>, String>;
    async fn save_policy(&self, policy: &RegisteredPolicyRule) -> Result<(), String>;
    async fn remove_policy(&self, name: &str) -> Result<(), String>;
    async fn load_policies(&self) -> Result<HashMap<String, RegisteredPolicyRule>, String>;
}
