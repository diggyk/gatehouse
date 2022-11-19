use std::collections::HashMap;

use crate::entity::RegisteredEntity;
use crate::group::RegisteredGroup;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::target::RegisteredTarget;

pub(crate) struct FileStorage {
    basepath: String,
}

impl FileStorage {
    pub async fn new(basepath: &str) -> Self {
        tokio::fs::create_dir_all(format!("{}/targets/", basepath))
            .await
            .expect("Could not create file backend storage");

        tokio::fs::create_dir_all(format!("{}/entities/", basepath))
            .await
            .expect("Could not create file backend storage");

        tokio::fs::create_dir_all(format!("{}/roles/", basepath))
            .await
            .expect("Could not create file backend storage");

        tokio::fs::create_dir_all(format!("{}/groups/", basepath))
            .await
            .expect("Could not create file backend storage");

        tokio::fs::create_dir_all(format!("{}/policies/", basepath))
            .await
            .expect("Could not create file backend storage");

        Self {
            basepath: basepath.to_string(),
        }
    }

    pub async fn save_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!(
            "{}/targets/{}-{}.json",
            self.basepath, tgt.typestr, tgt.name
        );

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        tokio::fs::write(target_path, json)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!(
            "{}/targets/{}-{}.json",
            self.basepath, tgt.typestr, tgt.name
        );

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String> {
        let mut targets = HashMap::new();

        let mut dir = tokio::fs::read_dir(format!("{}/targets", self.basepath))
            .await
            .expect("Could not read targets from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let json = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            let target: RegisteredTarget =
                serde_json::from_str(&json).map_err(|err| err.to_string())?;

            // get or create the hashmap for this "type" of target
            let typed_targets = targets
                .entry(target.typestr.clone())
                .or_insert_with(HashMap::new);

            typed_targets.insert(target.name.clone(), target.clone());

            println!("Loaded {}", target);
        }

        Ok(targets)
    }

    pub async fn save_entity(&self, tgt: &RegisteredEntity) -> Result<(), String> {
        let target_path = format!(
            "{}/entities/{}-{}.json",
            self.basepath, tgt.typestr, tgt.name
        );

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        tokio::fs::write(target_path, json)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_entity(&self, tgt: &RegisteredEntity) -> Result<(), String> {
        let target_path = format!(
            "{}/entities/{}-{}.json",
            self.basepath, tgt.typestr, tgt.name
        );

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_entities(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredEntity>>, String> {
        let mut targets = HashMap::new();

        let mut dir = tokio::fs::read_dir(format!("{}/entities", self.basepath))
            .await
            .expect("Could not read entities from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let json = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            let target: RegisteredEntity =
                serde_json::from_str(&json).map_err(|err| err.to_string())?;

            // get or create the hashmap for this "type" of target
            let typed_targets = targets
                .entry(target.typestr.clone())
                .or_insert_with(HashMap::new);

            typed_targets.insert(target.name.clone(), target.clone());

            println!("Loaded {}", target);
        }

        Ok(targets)
    }

    pub async fn save_role(&self, role: &RegisteredRole) -> Result<(), String> {
        let target_path = format!("{}/roles/{}.json", self.basepath, role.name);

        let json = serde_json::to_string(&role).map_err(|err| err.to_string())?;

        tokio::fs::write(target_path, json)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_role(&self, name: &str) -> Result<(), String> {
        let target_path = format!("{}/roles/{}.json", self.basepath, name);

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_roles(&self) -> Result<HashMap<String, RegisteredRole>, String> {
        let mut roles = HashMap::new();

        let mut dir = tokio::fs::read_dir(format!("{}/roles", self.basepath))
            .await
            .expect("Could not read entities from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let json = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            let role: RegisteredRole =
                serde_json::from_str(&json).map_err(|err| err.to_string())?;
            roles.insert(role.name.clone(), role.clone());

            println!(
                "Loaded role {} (used by {} groups)",
                role.name,
                role.groups.len()
            );
        }

        Ok(roles)
    }

    pub async fn save_group(&self, group: &RegisteredGroup) -> Result<(), String> {
        let target_path = format!("{}/groups/{}.json", self.basepath, group.name);

        let json = serde_json::to_string(&group).map_err(|err| err.to_string())?;

        tokio::fs::write(target_path, json)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_group(&self, name: &str) -> Result<(), String> {
        let target_path = format!("{}/groups/{}.json", self.basepath, name);

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_groups(&self) -> Result<HashMap<String, RegisteredGroup>, String> {
        let mut groups = HashMap::new();

        let mut dir = tokio::fs::read_dir(format!("{}/groups", self.basepath))
            .await
            .expect("Could not read entities from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let json = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            let group: RegisteredGroup =
                serde_json::from_str(&json).map_err(|err| err.to_string())?;
            groups.insert(group.name.clone(), group.clone());

            println!(
                "Loaded group {}: {} members  {} roles",
                group.name,
                group.members.len(),
                group.roles.len()
            );
        }

        Ok(groups)
    }

    pub async fn save_policy(&self, policy: &RegisteredPolicyRule) -> Result<(), String> {
        let target_path = format!("{}/policies/{}.json", self.basepath, policy.name);

        let json = serde_json::to_string(&policy).map_err(|err| err.to_string())?;

        tokio::fs::write(target_path, json)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_policy(&self, name: &str) -> Result<(), String> {
        let target_path = format!("{}/groups/{}.json", self.basepath, name);

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_policies(&self) -> Result<HashMap<String, RegisteredPolicyRule>, String> {
        let mut groups = HashMap::new();

        let mut dir = tokio::fs::read_dir(format!("{}/policies", self.basepath))
            .await
            .expect("Could not read policies from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let json = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            let policy: RegisteredPolicyRule =
                serde_json::from_str(&json).map_err(|err| err.to_string())?;
            groups.insert(policy.name.clone(), policy.clone());

            println!("Loaded policy {}", policy.name,);
        }

        Ok(groups)
    }
}
