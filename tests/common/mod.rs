use std::collections::HashMap;

use async_recursion::async_recursion;
use gatehouse::proto::common::AttributeValues;
use gatehouse::proto::entities::{
    AddEntityRequest, Entity, GetAllEntitiesRequest, ModifyEntityRequest, RemoveEntityRequest,
};
use gatehouse::proto::roles::{AddRoleRequest, GetAllRolesRequest, RemoveRoleRequest, Role};
use tokio::fs::read_dir;
use tokio::process::Command;
use tonic::transport::Channel;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;
use gatehouse::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};

pub fn str(s: &str) -> String {
    s.to_string()
}

pub fn to_attribs(attributes: Vec<(String, Vec<&str>)>) -> HashMap<String, AttributeValues> {
    attributes
        .into_iter()
        .map(|kv| {
            (
                kv.0,
                AttributeValues {
                    values: kv.1.into_iter().map(str).collect(),
                },
            )
        })
        .collect()
}

/// Adds a single target
pub async fn add_target(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    actions: Vec<&str>,
    attributes: Vec<(String, Vec<&str>)>,
) -> Target {
    let actions = actions.into_iter().map(str).collect();
    let attributes = to_attribs(attributes);

    client
        .add_target(AddTargetRequest {
            name: str(name),
            typestr: str(typestr),
            actions,
            attributes,
        })
        .await
        .expect("Failed to add target")
        .into_inner()
        .target
        .expect("No target returned after creation")
}

/// Modify a target
pub async fn modify_target(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    add_actions: Vec<&str>,
    add_attributes: Vec<(String, Vec<&str>)>,
    remove_actions: Vec<&str>,
    remove_attributes: Vec<(String, Vec<&str>)>,
) -> Target {
    let add_actions = add_actions.into_iter().map(str).collect();
    let add_attributes = to_attribs(add_attributes);
    let remove_actions = remove_actions.into_iter().map(str).collect();
    let remove_attributes = to_attribs(remove_attributes);

    client
        .modify_target(ModifyTargetRequest {
            name: str(name),
            typestr: str(typestr),
            add_actions,
            add_attributes,
            remove_actions,
            remove_attributes,
        })
        .await
        .expect("Failed to modify target")
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

/// Adds a single entity
pub async fn add_entity(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    attributes: Vec<(String, Vec<&str>)>,
) -> Entity {
    let attributes = to_attribs(attributes);

    client
        .add_entity(AddEntityRequest {
            name: str(name),
            typestr: str(typestr),
            attributes,
        })
        .await
        .expect("Failed to add entity")
        .into_inner()
        .entity
        .expect("No target returned after creation")
}

/// Modify a entity
pub async fn modify_entity(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    add_attributes: Vec<(String, Vec<&str>)>,
    remove_attributes: Vec<(String, Vec<&str>)>,
) -> Entity {
    let add_attributes = to_attribs(add_attributes);
    let remove_attributes = to_attribs(remove_attributes);

    client
        .modify_entity(ModifyEntityRequest {
            name: str(name),
            typestr: str(typestr),
            add_attributes,
            remove_attributes,
        })
        .await
        .expect("Failed to modify entity")
        .into_inner()
        .entity
        .expect("No entity returned after update")
}

/// Remove entity
pub async fn remove_entity(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
) -> Entity {
    client
        .remove_entity(RemoveEntityRequest {
            name: str(name),
            typestr: str(typestr),
        })
        .await
        .expect("Failed to add entity")
        .into_inner()
        .entity
        .expect("No entity returned after deletion")
}

/// Get all entities
pub async fn get_entities(
    client: &mut GatehouseClient<Channel>,
    name: Option<&str>,
    typestr: Option<&str>,
) -> Vec<Entity> {
    let name = name.map(str);
    let typestr = typestr.map(str);

    client
        .get_entities(GetAllEntitiesRequest { name, typestr })
        .await
        .expect("Failed to get all targets")
        .into_inner()
        .entities
}

/// Add a role
pub async fn add_role(client: &mut GatehouseClient<Channel>, name: &str) -> Role {
    client
        .add_role(AddRoleRequest { name: str(name) })
        .await
        .expect("Failed to add role")
        .into_inner()
        .role
        .expect("No target returned after creation")
}

/// Remove role
pub async fn remove_role(client: &mut GatehouseClient<Channel>, name: &str) -> Role {
    client
        .remove_role(RemoveRoleRequest { name: str(name) })
        .await
        .expect("Failed to remove role")
        .into_inner()
        .role
        .expect("No entity returned after deletion")
}

/// Get all roles
pub async fn get_roles(client: &mut GatehouseClient<Channel>, name: Option<&str>) -> Vec<Role> {
    let name = name.map(str);

    client
        .get_roles(GetAllRolesRequest { name })
        .await
        .expect("Failed to get all roles")
        .into_inner()
        .roles
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
