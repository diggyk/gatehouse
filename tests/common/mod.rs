use async_recursion::async_recursion;
use tokio::fs::read_dir;
use tokio::process::Command;
use tonic::transport::Channel;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;
use gatehouse::proto::targets::{
    AddTargetActionRequest, AddTargetRequest, GetAllTargetsRequest, RemoveTargetActionRequest,
    RemoveTargetRequest, Target,
};

pub fn str(s: &str) -> String {
    s.to_string()
}

/// Adds a single target
pub async fn add_target(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    actions: Vec<&str>,
) -> Target {
    let actions = actions.into_iter().map(str).collect();
    client
        .add_target(AddTargetRequest {
            name: str(name),
            typestr: str(typestr),
            actions,
        })
        .await
        .expect("Failed to add target")
        .into_inner()
        .target
        .expect("No target returned after creation")
}

/// Add actions to a target
pub async fn add_target_actions(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    actions: Vec<&str>,
) -> Target {
    let actions = actions.into_iter().map(str).collect();
    client
        .add_target_action(AddTargetActionRequest {
            name: str(name),
            typestr: str(typestr),
            actions,
        })
        .await
        .expect("Failed to add target")
        .into_inner()
        .target
        .expect("No target returned after update")
}

/// Remove actions from a target
pub async fn remove_target_actions(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    actions: Vec<&str>,
) -> Target {
    let actions = actions.into_iter().map(str).collect();
    client
        .remove_target_action(RemoveTargetActionRequest {
            name: str(name),
            typestr: str(typestr),
            actions,
        })
        .await
        .expect("Failed to add target")
        .into_inner()
        .target
        .expect("No target returned after update")
}

/// Remove target
pub async fn remove_target(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
) -> Target {
    client
        .remove_target(RemoveTargetRequest {
            name: str(name),
            typestr: str(typestr),
        })
        .await
        .expect("Failed to add target")
        .into_inner()
        .target
        .expect("No target returned after deletion")
}

/// Get all targets
pub async fn get_targets(
    client: &mut GatehouseClient<Channel>,
    name: Option<&str>,
    typestr: Option<&str>,
) -> Vec<Target> {
    let name = name.map(str);
    let typestr = typestr.map(str);

    client
        .get_targets(GetAllTargetsRequest { name, typestr })
        .await
        .expect("Failed to get all targets")
        .into_inner()
        .targets
}

#[async_recursion]
async fn clear_dir(path: &str) {
    let mut dir = read_dir(path).await.expect("Could not read tmp dir");

    while let Some(entry) = dir
        .next_entry()
        .await
        .expect("Could not read entry in tmp dir")
    {
        let metadata = entry
            .metadata()
            .await
            .expect("Could not read metadata for an entry in tmp dir");

        if metadata.is_file() {
            tokio::fs::remove_file(entry.path())
                .await
                .expect("Could not delete file in tmp dir");
        } else if metadata.is_dir() {
            if let Some(subpath) = entry.path().to_str() {
                clear_dir(subpath).await;
            }
        }
    }
}

/// Runs the server for us to test against. We should make sure this runs with a file backend
/// and that the local file storage is cleaned so it starts empty
pub async fn run_server() {
    clear_dir("/tmp/gatehouse").await;

    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("gatehouse-server")
        .kill_on_drop(true)
        .spawn()
        .expect("Could not start server")
        .wait()
        .await
        .expect("Server failed");
}
