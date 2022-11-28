use std::collections::HashMap;

use tonic::async_trait;

use crate::entity::RegisteredEntity;
use crate::group::RegisteredGroup;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::target::RegisteredTarget;

use super::Storage;

pub(crate) struct NilStorage;

#[async_trait]
impl Storage for NilStorage {
    async fn save_target(&self, _tgt: &RegisteredTarget) -> Result<(), String> {
        Ok(())
    }
    async fn remove_target(&self, _tgt: &RegisteredTarget) -> Result<(), String> {
        Ok(())
    }
    async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String> {
        Ok(HashMap::new())
    }
    async fn save_entity(&self, _tgt: &RegisteredEntity) -> Result<(), String> {
        Ok(())
    }
    async fn remove_entity(&self, _tgt: &RegisteredEntity) -> Result<(), String> {
        Ok(())
    }
    async fn load_entities(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredEntity>>, String> {
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
}
