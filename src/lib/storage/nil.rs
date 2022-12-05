use std::collections::HashMap;

use tonic::async_trait;

use crate::actor::RegisteredActor;
use crate::group::RegisteredGroup;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::target::RegisteredTarget;

use super::{BackendUpdate, Storage};

pub(crate) struct NilStorage;

#[async_trait]
impl Storage for NilStorage {
    async fn save_target(&self, _tgt: &RegisteredTarget) -> Result<(), String> {
        Ok(())
    }
    async fn remove_target(&self, _typestr: &str, _name: &str) -> Result<(), String> {
        Ok(())
    }
    async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String> {
        Ok(HashMap::new())
    }
    async fn save_actor(&self, _tgt: &RegisteredActor) -> Result<(), String> {
        Ok(())
    }
    async fn remove_actor(&self, _typestr: &str, _name: &str) -> Result<(), String> {
        Ok(())
    }
    async fn load_actors(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredActor>>, String> {
        Ok(HashMap::new())
    }
    async fn save_role(&self, _role: &RegisteredRole) -> Result<(), String> {
        Ok(())
    }
    async fn remove_role(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    async fn load_roles(&self) -> Result<HashMap<String, RegisteredRole>, String> {
        Ok(HashMap::new())
    }
    async fn save_group(&self, _group: &RegisteredGroup) -> Result<(), String> {
        Ok(())
    }
    async fn remove_group(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    async fn load_groups(&self) -> Result<HashMap<String, RegisteredGroup>, String> {
        Ok(HashMap::new())
    }
    async fn save_policy(&self, _policy: &RegisteredPolicyRule) -> Result<(), String> {
        Ok(())
    }
    async fn remove_policy(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    async fn load_policies(&self) -> Result<HashMap<String, RegisteredPolicyRule>, String> {
        Ok(HashMap::new())
    }
    async fn persist_changes(&self, _updates: &[BackendUpdate]) -> Result<(), String> {
        Ok(())
    }
}
