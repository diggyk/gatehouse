//! Helpers to quickly create requests and use a client to talk to the server

use std::collections::HashMap;

use crate::proto::actors::{
    Actor, AddActorRequest, GetActorsRequest, ModifyActorRequest, RemoveActorRequest,
};
use crate::proto::common::AttributeValues;
use crate::proto::groups::{
    AddGroupRequest, GetGroupsRequest, Group, GroupMember, ModifyGroupRequest, RemoveGroupRequest,
};
use crate::proto::policies::{
    ActorCheck, AddPolicyRequest, Decide, GetPoliciesRequest, KvCheck, ModifyPolicyRequest,
    PolicyRule, RemovePolicyRequest, TargetCheck,
};
use crate::proto::roles::{AddRoleRequest, GetRolesRequest, RemoveRoleRequest, Role};
use tonic::transport::Channel;

use crate::proto::base::gatehouse_client::GatehouseClient;
use crate::proto::targets::{
    AddTargetRequest, GetTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};

/// Helper to quickly create a string
pub fn str(s: &str) -> String {
    s.to_string()
}

/// Quickly create a hashmap of attribute names and attribute values
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
) -> Result<Target, String> {
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
        .map_err(|err| format!("Failed to add target: {err}"))?
        .into_inner()
        .target
        .ok_or_else(|| str("No target returned after creation"))
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
) -> Result<Target, String> {
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
        .map_err(|err| format!("Failed to modify target: {err}"))?
        .into_inner()
        .target
        .ok_or_else(|| str("No target returned after update"))
}

/// Remove target
pub async fn remove_target(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
) -> Result<Target, String> {
    client
        .remove_target(RemoveTargetRequest {
            name: str(name),
            typestr: str(typestr),
        })
        .await
        .map_err(|err| format!("Failed to remove target: {err}"))?
        .into_inner()
        .target
        .ok_or_else(|| str("No target returned after deletion"))
}

/// Get all targets
pub async fn get_targets<S: Into<String>>(
    client: &mut GatehouseClient<Channel>,
    name: Option<S>,
    typestr: Option<S>,
) -> Result<Vec<Target>, String> {
    let name = name.map(|str| str.into());
    let typestr = typestr.map(|str| str.into());

    Ok(client
        .get_targets(GetTargetsRequest { name, typestr })
        .await
        .map_err(|err| format!("Failed to get targets: {err}"))?
        .into_inner()
        .targets)
}

/// Adds a single actor
pub async fn add_actor(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    attributes: Vec<(String, Vec<&str>)>,
) -> Result<Actor, String> {
    let attributes = to_attribs(attributes);

    client
        .add_actor(AddActorRequest {
            name: str(name),
            typestr: str(typestr),
            attributes,
        })
        .await
        .map_err(|err| format!("Failed to add actor: {err}"))?
        .into_inner()
        .actor
        .ok_or_else(|| str("No target returned after creation"))
}

/// Modify a actor
pub async fn modify_actor(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
    add_attributes: Vec<(String, Vec<&str>)>,
    remove_attributes: Vec<(String, Vec<&str>)>,
) -> Result<Actor, String> {
    let add_attributes = to_attribs(add_attributes);
    let remove_attributes = to_attribs(remove_attributes);

    client
        .modify_actor(ModifyActorRequest {
            name: str(name),
            typestr: str(typestr),
            add_attributes,
            remove_attributes,
        })
        .await
        .map_err(|err| format!("Failed to modify actor: {err}"))?
        .into_inner()
        .actor
        .ok_or_else(|| str("No actor returned after update"))
}

/// Remove actor
pub async fn remove_actor(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    typestr: &str,
) -> Result<Actor, String> {
    client
        .remove_actor(RemoveActorRequest {
            name: str(name),
            typestr: str(typestr),
        })
        .await
        .map_err(|err| format!("Failed to remove actor: {err}"))?
        .into_inner()
        .actor
        .ok_or_else(|| str("No actor returned after deletion"))
}

/// Get all actors
pub async fn get_actors<S: Into<String>>(
    client: &mut GatehouseClient<Channel>,
    name: Option<S>,
    typestr: Option<S>,
) -> Result<Vec<Actor>, String> {
    let name = name.map(|s| s.into());
    let typestr = typestr.map(|s| s.into());

    Ok(client
        .get_actors(GetActorsRequest { name, typestr })
        .await
        .map_err(|err| format!("Failed to get actors: {err}"))?
        .into_inner()
        .actors)
}

/// Add a role
pub async fn add_role(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    desc: Option<String>,
    groups: Vec<String>,
) -> Result<Role, String> {
    client
        .add_role(AddRoleRequest {
            name: str(name),
            desc,
            granted_to: groups,
        })
        .await
        .map_err(|err| format!("Failed to add role: {err}"))?
        .into_inner()
        .role
        .ok_or_else(|| str("No target returned after creation"))
}

/// Remove role
pub async fn remove_role(
    client: &mut GatehouseClient<Channel>,
    name: &str,
) -> Result<Role, String> {
    client
        .remove_role(RemoveRoleRequest { name: str(name) })
        .await
        .map_err(|err| format!("Failed to remove role: {err}"))?
        .into_inner()
        .role
        .ok_or_else(|| str("No role returned after deletion"))
}

/// Get all roles
pub async fn get_roles(
    client: &mut GatehouseClient<Channel>,
    name: Option<&str>,
) -> Result<Vec<Role>, String> {
    let name = name.map(str);

    Ok(client
        .get_roles(GetRolesRequest { name })
        .await
        .map_err(|err| format!("Failed to get roles: {err}"))?
        .into_inner()
        .roles)
}

/// Add a group
pub async fn add_group(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    desc: Option<&str>,
    members: Vec<(&str, &str)>,
    roles: Vec<&str>,
) -> Result<Group, String> {
    let members: Vec<GroupMember> = members
        .iter()
        .map(|(n, t)| GroupMember {
            name: n.to_string(),
            typestr: t.to_string(),
        })
        .collect();
    let roles = roles.iter().map(|r| r.to_string()).collect();

    let req = AddGroupRequest {
        name: name.to_string(),
        desc: desc.map(String::from),
        members,
        roles,
    };

    client
        .add_group(req)
        .await
        .map_err(|err| format!("Failed to add group: {err}"))?
        .into_inner()
        .group
        .ok_or_else(|| str("No group in add group response"))
}

/// Modify a group
pub async fn modify_group(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    desc: Option<&str>,
    add_members: Vec<(&str, &str)>,
    add_roles: Vec<&str>,
    remove_members: Vec<(&str, &str)>,
    remove_roles: Vec<&str>,
) -> Result<Group, String> {
    let add_members: Vec<GroupMember> = add_members
        .iter()
        .map(|(n, t)| GroupMember {
            name: n.to_string(),
            typestr: t.to_string(),
        })
        .collect();
    let add_roles = add_roles.iter().map(|r| r.to_string()).collect();

    let remove_members: Vec<GroupMember> = remove_members
        .iter()
        .map(|(n, t)| GroupMember {
            name: n.to_string(),
            typestr: t.to_string(),
        })
        .collect();
    let remove_roles = remove_roles.iter().map(|r| r.to_string()).collect();

    let req = ModifyGroupRequest {
        name: name.to_string(),
        desc: desc.map(String::from),
        add_members,
        add_roles,
        remove_members,
        remove_roles,
    };

    client
        .modify_group(req)
        .await
        .map_err(|err| format!("Failed to modify group: {err}"))?
        .into_inner()
        .group
        .ok_or_else(|| str("Did not get group after modifications"))
}

/// Remove group
pub async fn remove_group(
    client: &mut GatehouseClient<Channel>,
    name: &str,
) -> Result<Group, String> {
    client
        .remove_group(RemoveGroupRequest {
            name: name.to_string(),
        })
        .await
        .map_err(|err| format!("Failed to remove group: {err}"))?
        .into_inner()
        .group
        .ok_or_else(|| str("Did not get returned group after removal"))
}

/// Get groups
pub async fn get_groups(
    client: &mut GatehouseClient<Channel>,
    name: Option<&str>,
    member: Option<(&str, &str)>,
    role: Option<&str>,
) -> Result<Vec<Group>, String> {
    let name = name.map(String::from);
    let member = member.map(|(name, typestr)| GroupMember {
        name: name.to_string(),
        typestr: typestr.to_string(),
    });
    let role = role.map(String::from);

    Ok(client
        .get_groups(GetGroupsRequest { name, member, role })
        .await
        .map_err(|err| format!("Failed to get groups: {err}"))?
        .into_inner()
        .groups)
}

/// Add a policy
pub async fn add_policy(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    desc: Option<&str>,
    actor_check: Option<ActorCheck>,
    env_attributes: Vec<KvCheck>,
    target_check: Option<TargetCheck>,
    decision: Decide,
) -> Result<PolicyRule, String> {
    let rule = PolicyRule {
        name: name.to_string(),
        desc: desc.map(String::from),
        actor_check,
        env_attributes,
        target_check,
        decision: decision.into(),
    };
    client
        .add_policy(AddPolicyRequest { rule: Some(rule) })
        .await
        .map_err(|err| format!("Failed to add policy: {err}"))?
        .into_inner()
        .rule
        .ok_or_else(|| str("No policy returned after creation"))
}

/// Modify/replace an existing policy
pub async fn modify_policy(
    client: &mut GatehouseClient<Channel>,
    name: &str,
    desc: Option<&str>,
    actor_check: Option<ActorCheck>,
    env_attributes: Vec<KvCheck>,
    target_check: Option<TargetCheck>,
    decision: Decide,
) -> Result<PolicyRule, String> {
    let rule = PolicyRule {
        name: name.to_string(),
        desc: desc.map(String::from),
        actor_check,
        env_attributes,
        target_check,
        decision: decision.into(),
    };
    client
        .modify_policy(ModifyPolicyRequest { rule: Some(rule) })
        .await
        .map_err(|err| format!("Failed to modify policy: {err}"))?
        .into_inner()
        .rule
        .ok_or_else(|| str("No policy returned after update"))
}

/// Remove an existing policy
pub async fn remove_policy(
    client: &mut GatehouseClient<Channel>,
    name: &str,
) -> Result<PolicyRule, String> {
    client
        .remove_policy(RemovePolicyRequest {
            name: name.to_string(),
        })
        .await
        .map_err(|err| format!("Failed to remove policy: {err}"))?
        .into_inner()
        .rule
        .ok_or_else(|| str("No policy returned after removal"))
}

/// Search for policies
pub async fn get_policies(
    client: &mut GatehouseClient<Channel>,
    name: Option<&str>,
) -> Result<Vec<PolicyRule>, String> {
    let req = GetPoliciesRequest {
        name: name.map(String::from),
    };

    Ok(client
        .get_policies(req)
        .await
        .map_err(|err| format!("Failed to get policies: {err}"))?
        .into_inner()
        .rules)
}
