use std::collections::{HashMap, HashSet};

use crate::entity::RegisteredEntity;
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

        Self {
            basepath: basepath.to_string(),
        }
    }

    pub async fn save_target(&self, tgt: RegisteredTarget) -> Result<(), String> {
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

    pub async fn remove_target(&self, tgt: RegisteredTarget) -> Result<(), String> {
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

    pub async fn save_entity(&self, tgt: RegisteredEntity) -> Result<(), String> {
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

    pub async fn remove_entity(&self, tgt: RegisteredEntity) -> Result<(), String> {
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

    pub async fn save_role(&self, role: &str) -> Result<(), String> {
        let target_path = format!("{}/roles/{}", self.basepath, role);

        tokio::fs::write(target_path, role)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn remove_role(&self, role: &str) -> Result<(), String> {
        let target_path = format!("{}/roles/{}", self.basepath, role);

        tokio::fs::remove_file(target_path)
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn load_roles(&self) -> Result<HashSet<String>, String> {
        let mut roles = HashSet::new();

        let mut dir = tokio::fs::read_dir(format!("{}/roles", self.basepath))
            .await
            .expect("Could not read entities from filesystem");

        while let Some(entry) = dir.next_entry().await.map_err(|err| err.to_string())? {
            let name = tokio::fs::read_to_string(entry.path())
                .await
                .map_err(|err| err.to_string())?;

            roles.insert(name.clone());

            println!("Loaded role {}", name);
        }

        Ok(roles)
    }
}
