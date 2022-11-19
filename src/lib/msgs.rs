#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::entities::{
    AddEntityRequest, Entity, GetAllEntitiesRequest, ModifyEntityRequest, RemoveEntityRequest,
};
use crate::proto::groups::{
    AddGroupRequest, GetAllGroupsRequest, Group, ModifyGroupRequest, RemoveGroupRequest,
};
use crate::proto::policies::{
    AddPolicyRequest, GetPoliciesRequest, ModifyPolicyRequest, PolicyRule, RemovePolicyRequest,
};
use crate::proto::roles::{AddRoleRequest, GetAllRolesRequest, RemoveRoleRequest, Role};
use crate::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};

#[derive(Debug)]
pub(crate) enum DsRequest {
    AddTarget(AddTargetRequest, Sender<DsResponse>),
    ModifyTarget(ModifyTargetRequest, Sender<DsResponse>),
    RemoveTarget(RemoveTargetRequest, Sender<DsResponse>),
    GetTargets(GetAllTargetsRequest, Sender<DsResponse>),

    AddEntity(AddEntityRequest, Sender<DsResponse>),
    ModifyEntity(ModifyEntityRequest, Sender<DsResponse>),
    RemoveEntity(RemoveEntityRequest, Sender<DsResponse>),
    GetEntities(GetAllEntitiesRequest, Sender<DsResponse>),

    AddRole(AddRoleRequest, Sender<DsResponse>),
    RemoveRole(RemoveRoleRequest, Sender<DsResponse>),
    GetRoles(GetAllRolesRequest, Sender<DsResponse>),

    AddGroup(AddGroupRequest, Sender<DsResponse>),
    ModifyGroup(ModifyGroupRequest, Sender<DsResponse>),
    RemoveGroup(RemoveGroupRequest, Sender<DsResponse>),
    GetGroups(GetAllGroupsRequest, Sender<DsResponse>),

    AddPolicy(AddPolicyRequest, Sender<DsResponse>),
    ModifyPolicy(ModifyPolicyRequest, Sender<DsResponse>),
    RemovePolicy(RemovePolicyRequest, Sender<DsResponse>),
    GetPolicies(GetPoliciesRequest, Sender<DsResponse>),
}

#[derive(Debug)]
pub enum DsResponse {
    Error(Status),

    SingleTarget(Target),
    MultipleTargets(Vec<Target>),

    SingleEntity(Entity),
    MultipleEntities(Vec<Entity>),

    SingleRole(Role),
    MultipleRoles(Vec<Role>),

    SingleGroup(Group),
    MultipleGroups(Vec<Group>),

    SinglePolicy(PolicyRule),
    MultiplePolicies(Vec<PolicyRule>),
}
