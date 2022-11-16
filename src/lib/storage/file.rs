use std::collections::HashMap;

use crate::target::RegisteredTarget;

pub(crate) struct FileStorage {
    basepath: String,
}

impl FileStorage {
    pub async fn new(basepath: &str) -> Self {
        tokio::fs::create_dir_all(format!("{}/targets/", basepath))
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
}
