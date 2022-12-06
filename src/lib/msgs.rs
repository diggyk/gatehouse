#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::actors::{
    Actor, AddActorRequest, GetActorsRequest, ModifyActorRequest, RemoveActorRequest,
};
use crate::proto::base::CheckRequest;
use crate::proto::groups::{
    AddGroupRequest, GetGroupsRequest, Group, ModifyGroupRequest, RemoveGroupRequest,
};
use crate::proto::policies::{
    AddPolicyRequest, Decide, GetPoliciesRequest, ModifyPolicyRequest, PolicyRule,
    RemovePolicyRequest,
};
use crate::proto::roles::{
    AddRoleRequest, GetRolesRequest, ModifyRoleRequest, RemoveRoleRequest, Role,
};
use crate::proto::targets::{
    AddTargetRequest, GetTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};
use crate::storage::BackendUpdate;

#[derive(Debug)]
pub(crate) enum DsRequest {
    AddTarget(AddTargetRequest, Sender<DsResponse>),
    ModifyTarget(ModifyTargetRequest, Sender<DsResponse>),
    RemoveTarget(RemoveTargetRequest, Sender<DsResponse>),
    GetTargets(GetTargetsRequest, Sender<DsResponse>),

    AddActor(AddActorRequest, Sender<DsResponse>),
    ModifyActor(ModifyActorRequest, Sender<DsResponse>),
    RemoveActor(RemoveActorRequest, Sender<DsResponse>),
    GetActors(GetActorsRequest, Sender<DsResponse>),

    AddRole(AddRoleRequest, Sender<DsResponse>),
    ModifyRole(ModifyRoleRequest, Sender<DsResponse>),
    RemoveRole(RemoveRoleRequest, Sender<DsResponse>),
    GetRoles(GetRolesRequest, Sender<DsResponse>),

    AddGroup(AddGroupRequest, Sender<DsResponse>),
    ModifyGroup(ModifyGroupRequest, Sender<DsResponse>),
    RemoveGroup(RemoveGroupRequest, Sender<DsResponse>),
    GetGroups(GetGroupsRequest, Sender<DsResponse>),

    AddPolicy(AddPolicyRequest, Sender<DsResponse>),
    ModifyPolicy(ModifyPolicyRequest, Sender<DsResponse>),
    RemovePolicy(RemovePolicyRequest, Sender<DsResponse>),
    GetPolicies(GetPoliciesRequest, Sender<DsResponse>),

    Check(CheckRequest, Sender<DsResponse>),
    Update(BackendUpdate),
}

#[derive(Debug)]
pub enum DsResponse {
    Error(Status),

    SingleTarget(Target),
    MultipleTargets(Vec<Target>),

    SingleActor(Actor),
    MultipleActors(Vec<Actor>),

    SingleRole(Role),
    MultipleRoles(Vec<Role>),

    SingleGroup(Group),
    MultipleGroups(Vec<Group>),

    SinglePolicy(Box<PolicyRule>),
    MultiplePolicies(Vec<PolicyRule>),

    CheckResult(Decide),
}
