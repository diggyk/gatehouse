use std::collections::HashMap;
use std::process::exit;

use etcd_client::{Client, GetOptions};
use tonic::async_trait;

use crate::entity::RegisteredEntity;
use crate::group::RegisteredGroup;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::target::RegisteredTarget;

use super::Storage;

fn econv<T: std::fmt::Display>(err: T) -> String {
    err.to_string()
}

pub(crate) struct EtcdStorage {
    basepath: String,
    client: Client,
}

impl EtcdStorage {
    pub async fn new(url: &str) -> Self {
        let mut client = match Client::connect([url], None).await {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Could not connect to Etcd storage: {err}");
                exit(1);
            }
        };

        let basepath = String::from("/gatehouse");

        if let Err(err) = client
            .get(basepath.as_bytes(), Some(GetOptions::new()))
            .await
        {
            eprintln!("Failed to query Etcd on startup: {err}");
            exit(1);
        }

        Self { basepath, client }
    }
}

#[async_trait]
impl Storage for EtcdStorage {
    async fn save_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!("{}/targets/{}/{}", self.basepath, tgt.typestr, tgt.name);

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(target_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }

    async fn remove_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!("{}/targets/{}/{}", self.basepath, tgt.typestr, tgt.name);

        self.client
            .kv_client()
            .delete(target_path, None)
            .await
            .map_err(econv)?;
        Ok(())
    }

    async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String> {
        let targets_path = format!("{}/targets", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(targets_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let target: RegisteredTarget = serde_json::from_str(val).map_err(econv)?;
            let typed_targets = map
                .entry(target.typestr.clone())
                .or_insert_with(HashMap::new);
            typed_targets.insert(target.name.clone(), target);
        }

        Ok(map)
    }
    async fn save_entity(&self, tgt: &RegisteredEntity) -> Result<(), String> {
        let entity_path = format!("{}/entities/{}/{}", self.basepath, tgt.typestr, tgt.name);

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(entity_path, json, None)
            .await
            .map_err(econv)?;
        Ok(())
    }
    async fn remove_entity(&self, tgt: &RegisteredEntity) -> Result<(), String> {
        let entity_path = format!("{}/entities/{}/{}", self.basepath, tgt.typestr, tgt.name);

        self.client
            .kv_client()
            .delete(entity_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_entities(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredEntity>>, String> {
        let entities_path = format!("{}/entities", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(entities_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let entity: RegisteredEntity = serde_json::from_str(val).map_err(econv)?;
            let typed_entities = map
                .entry(entity.typestr.clone())
                .or_insert_with(HashMap::new);
            typed_entities.insert(entity.name.clone(), entity);
        }

        Ok(map)
    }
    async fn save_role(&self, role: &RegisteredRole) -> Result<(), String> {
        let role_path = format!("{}/roles/{}", self.basepath, role.name);

        let json = serde_json::to_string(&role).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(role_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_role(&self, name: &str) -> Result<(), String> {
        let role_path = format!("{}/roles/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(role_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_roles(&self) -> Result<HashMap<String, RegisteredRole>, String> {
        let roles_path = format!("{}/roles", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(roles_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let role: RegisteredRole = serde_json::from_str(val).map_err(econv)?;
            map.insert(role.name.clone(), role);
        }

        Ok(map)
    }
    async fn save_group(&self, group: &RegisteredGroup) -> Result<(), String> {
        let group_path = format!("{}/groups/{}", self.basepath, group.name);

        let json = serde_json::to_string(&group).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(group_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_group(&self, name: &str) -> Result<(), String> {
        let group_path = format!("{}/groups/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(group_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_groups(&self) -> Result<HashMap<String, RegisteredGroup>, String> {
        let groups_path = format!("{}/groups", self.basepath);

        let response = self
            .client
            .kv_client()
            .get(groups_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in response.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let group: RegisteredGroup = serde_json::from_str(val).map_err(econv)?;
            map.insert(group.name.clone(), group);
        }

        Ok(map)
    }
    async fn save_policy(&self, policy: &RegisteredPolicyRule) -> Result<(), String> {
        let policy_path = format!("{}/policies/{}", self.basepath, policy.name);

        let json = serde_json::to_string(&policy).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(policy_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_policy(&self, name: &str) -> Result<(), String> {
        let policy_path = format!("{}/policies/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(policy_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_policies(&self) -> Result<HashMap<String, RegisteredPolicyRule>, String> {
        let policies_path = format!("{}/policies", self.basepath);

        let response = self
            .client
            .kv_client()
            .get(policies_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in response.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let policy: RegisteredPolicyRule = serde_json::from_str(val).map_err(econv)?;
            map.insert(policy.name.clone(), policy);
        }

        Ok(map)
    }
}
