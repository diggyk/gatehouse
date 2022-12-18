//! The main Gatehouse server binary

use flume::Sender;
use tokio::sync::oneshot::{channel, Receiver};
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

use crate::ds::Datastore;
use crate::msgs::{DsRequest, DsResponse};
use crate::proto::actors::{
    ActorResponse, AddActorRequest, GetActorsRequest, ModifyActorRequest, MultiActorResponse,
    RemoveActorRequest,
};
use crate::proto::base::gatehouse_server::Gatehouse;
use crate::proto::base::{CheckRequest, CheckResponse};
use crate::proto::groups::{
    AddGroupRequest, GetGroupsRequest, GroupResponse, ModifyGroupRequest, MultiGroupResponse,
    RemoveGroupRequest,
};
use crate::proto::policies::{
    AddPolicyRequest, GetPoliciesRequest, ModifyPolicyRequest, MultiPolicyResponse, PolicyResponse,
    RemovePolicyRequest,
};
use crate::proto::roles::{
    AddRoleRequest, GetRolesRequest, ModifyRoleRequest, MultiRoleResponse, RemoveRoleRequest,
    RoleResponse,
};
use crate::proto::targets::{
    AddTargetRequest, GetTargetsRequest, ModifyTargetRequest, MultiTargetResponse,
    RemoveTargetRequest, TargetResponse,
};
use crate::StorageType;

#[derive(Debug)]
/// The core Gatehouse server
pub struct GatehouseSvc {
    dstx: Sender<DsRequest>,
}

impl GatehouseSvc {
    /// Create a new Gatehouse service
    pub async fn new(storage: &StorageType) -> Self {
        let dstx = Datastore::create(storage).await;
        GatehouseSvc { dstx }
    }
}

impl GatehouseSvc {
    /// Wait for a response from the datastore
    async fn call_datastore(
        &self,
        req: DsRequest,
        op: &str,
        rx: Receiver<DsResponse>,
    ) -> Result<DsResponse, Status> {
        if let Err(err) = self.dstx.send_async(req).await {
            // TODO! -- add metrics
            eprintln!("{} failed: {:?}", op, err);
            return Err(Status::internal(err.to_string()));
        }
        tokio::select! {
            _ = sleep(Duration::from_secs(30)) => {
                // TODO! -- add metrics
                eprintln!("Timeout waiting of response");
                Err(Status::deadline_exceeded("Timeout waiting for response from datastore"))
            },
            msg = rx => {
                let msg = msg.map_err(|err| Status::internal(err.to_string()))?;
                Ok(msg)
            },
        }
    }
}

#[tonic::async_trait]
impl Gatehouse for GatehouseSvc {
    //** TARGETS  **//

    /// Add a new target
    async fn add_target(
        &self,
        request: Request<AddTargetRequest>,
    ) -> Result<Response<TargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        println!("svc: Add {}/{}", req.typestr, req.name);
        if req.typestr.is_empty() || req.name.is_empty() {
            return Err(Status::invalid_argument("Name and typestr cannot be null"));
        }

        match self
            .call_datastore(DsRequest::AddTarget(req.clone(), tx), "add target", rx)
            .await?
        {
            DsResponse::SingleTarget(tgt) => {
                //TODO! -- add metrics
                println!("Added target {}", tgt);
                return Ok(Response::new(TargetResponse { target: Some(tgt) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify an existing target
    async fn modify_target(
        &self,
        request: Request<ModifyTargetRequest>,
    ) -> Result<Response<TargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::ModifyTarget(req.clone(), tx),
                "modify target",
                rx,
            )
            .await?
        {
            DsResponse::SingleTarget(tgt) => {
                //TODO! -- add metrics
                println!("Updated target: {}", tgt);
                return Ok(Response::new(TargetResponse { target: Some(tgt) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Remove an existing target
    async fn remove_target(
        &self,
        request: Request<RemoveTargetRequest>,
    ) -> Result<Response<TargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::RemoveTarget(req.clone(), tx),
                "remove target",
                rx,
            )
            .await?
        {
            DsResponse::SingleTarget(tgt) => {
                //TODO! -- add metrics
                println!("Removed target {}", tgt);
                return Ok(Response::new(TargetResponse { target: Some(tgt) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get all targets
    async fn get_targets(
        &self,
        request: Request<GetTargetsRequest>,
    ) -> Result<Response<MultiTargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetTargets(req.clone(), tx), "get target(s)", rx)
            .await?
        {
            DsResponse::MultipleTargets(tgts) => {
                //TODO! -- add metrics
                println!("Got {} targets", tgts.len());
                return Ok(Response::new(MultiTargetResponse { targets: tgts }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    //** ENTITIES **//

    /// Add an actor
    async fn add_actor(
        &self,
        request: Request<AddActorRequest>,
    ) -> Result<Response<ActorResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        // wait for the datastore to respond
        match self
            .call_datastore(DsRequest::AddActor(req.clone(), tx), "add actor", rx)
            .await?
        {
            DsResponse::SingleActor(actor) => {
                //TODO! -- add metrics
                println!("Added actor {}", actor);
                Ok(Response::new(ActorResponse { actor: Some(actor) }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify an actor
    async fn modify_actor(
        &self,
        request: Request<ModifyActorRequest>,
    ) -> Result<Response<ActorResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::ModifyActor(req.clone(), tx), "modify actor", rx)
            .await?
        {
            DsResponse::SingleActor(actor) => {
                //TODO! -- add metrics
                println!("Modify actor {}", actor);
                Ok(Response::new(ActorResponse { actor: Some(actor) }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Remove an actor
    async fn remove_actor(
        &self,
        request: Request<RemoveActorRequest>,
    ) -> Result<Response<ActorResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::RemoveActor(req.clone(), tx), "remove actor", rx)
            .await?
        {
            DsResponse::SingleActor(actor) => {
                //TODO! -- add metrics
                println!("Remove actor {}", actor);
                Ok(Response::new(ActorResponse { actor: Some(actor) }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get all entries
    async fn get_actors(
        &self,
        request: Request<GetActorsRequest>,
    ) -> Result<Response<MultiActorResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetActors(req.clone(), tx), "get actors", rx)
            .await?
        {
            DsResponse::MultipleActors(actors) => {
                //TODO! -- add metrics
                println!("Got {} actors", actors.len());
                return Ok(Response::new(MultiActorResponse { actors }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    //** ROLES **//

    /// Add a role
    async fn add_role(
        &self,
        request: Request<AddRoleRequest>,
    ) -> Result<Response<RoleResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::AddRole(req.clone(), tx), "add role", rx)
            .await?
        {
            DsResponse::SingleRole(role) => {
                //TODO! -- add metrics
                println!("Added role {}", role);
                return Ok(Response::new(RoleResponse { role: Some(role) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify a role
    async fn modify_role(
        &self,
        request: Request<ModifyRoleRequest>,
    ) -> Result<Response<RoleResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::ModifyRole(req.clone(), tx), "modify role", rx)
            .await?
        {
            DsResponse::SingleRole(role) => {
                //TODO! -- add metrics
                println!("Modified role {}", role);
                return Ok(Response::new(RoleResponse { role: Some(role) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Delete a role
    async fn remove_role(
        &self,
        request: Request<RemoveRoleRequest>,
    ) -> Result<Response<RoleResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::RemoveRole(req.clone(), tx), "remove role", rx)
            .await?
        {
            DsResponse::SingleRole(role) => {
                //TODO! -- add metrics
                println!("Removed role {}", role);
                return Ok(Response::new(RoleResponse { role: Some(role) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get all roles (or a specific one by name)
    async fn get_roles(
        &self,
        request: Request<GetRolesRequest>,
    ) -> Result<Response<MultiRoleResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetRoles(req.clone(), tx), "get roles", rx)
            .await?
        {
            DsResponse::MultipleRoles(roles) => {
                //TODO! -- add metrics
                println!("Get {} roles", roles.len());
                return Ok(Response::new(MultiRoleResponse { roles }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    //** GROUPS **//

    /// Add a group
    async fn add_group(
        &self,
        request: Request<AddGroupRequest>,
    ) -> Result<Response<GroupResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::AddGroup(req.clone(), tx), "add group", rx)
            .await?
        {
            DsResponse::SingleGroup(group) => {
                //TODO! -- add metrics
                println!("Added group {}", group);
                return Ok(Response::new(GroupResponse { group: Some(group) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify a group
    async fn modify_group(
        &self,
        request: Request<ModifyGroupRequest>,
    ) -> Result<Response<GroupResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::ModifyGroup(req.clone(), tx), "modify group", rx)
            .await?
        {
            DsResponse::SingleGroup(group) => {
                //TODO! -- add metrics
                println!("Modified group {}", group);
                return Ok(Response::new(GroupResponse { group: Some(group) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Remove a group
    async fn remove_group(
        &self,
        request: Request<RemoveGroupRequest>,
    ) -> Result<Response<GroupResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::RemoveGroup(req.clone(), tx), "remove group", rx)
            .await?
        {
            DsResponse::SingleGroup(group) => {
                //TODO! -- add metrics
                println!("Removed group {}", group);
                return Ok(Response::new(GroupResponse { group: Some(group) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get groups (with optional filters)
    async fn get_groups(
        &self,
        request: Request<GetGroupsRequest>,
    ) -> Result<Response<MultiGroupResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetGroups(req.clone(), tx), "get groups", rx)
            .await?
        {
            DsResponse::MultipleGroups(groups) => {
                //TODO! -- add metrics
                println!("Got {} groups", groups.len());
                return Ok(Response::new(MultiGroupResponse { groups }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Add policy
    async fn add_policy(
        &self,
        request: Request<AddPolicyRequest>,
    ) -> Result<Response<PolicyResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::AddPolicy(req.clone(), tx), "add policy", rx)
            .await?
        {
            DsResponse::SinglePolicy(rule) => {
                //TODO! -- add metrics
                println!("Added policy rule {}", rule);
                return Ok(Response::new(PolicyResponse { rule: Some(*rule) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify policy
    async fn modify_policy(
        &self,
        request: Request<ModifyPolicyRequest>,
    ) -> Result<Response<PolicyResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::ModifyPolicy(req.clone(), tx),
                "modify policy",
                rx,
            )
            .await?
        {
            DsResponse::SinglePolicy(rule) => {
                //TODO! -- add metrics
                println!("Modified policy rule {}", rule);
                return Ok(Response::new(PolicyResponse { rule: Some(*rule) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Remove policy
    async fn remove_policy(
        &self,
        request: Request<RemovePolicyRequest>,
    ) -> Result<Response<PolicyResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::RemovePolicy(req.clone(), tx),
                "remove policy",
                rx,
            )
            .await?
        {
            DsResponse::SinglePolicy(rule) => {
                //TODO! -- add metrics
                println!("Removed policy rule {}", rule);
                return Ok(Response::new(PolicyResponse { rule: Some(*rule) }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get policies based on filters
    async fn get_policies(
        &self,
        request: Request<GetPoliciesRequest>,
    ) -> Result<Response<MultiPolicyResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetPolicies(req.clone(), tx), "get policies", rx)
            .await?
        {
            DsResponse::MultiplePolicies(rules) => {
                //TODO! -- add metrics
                println!("Got {} policies", rules.len());
                return Ok(Response::new(MultiPolicyResponse { rules }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Make a decision an actor wanting to take an action on a target
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        if req.actor.is_none() {
            return Err(Status::invalid_argument("Actor cannot be null"));
        }

        match self
            .call_datastore(DsRequest::Check(req.clone(), tx), "perform check", rx)
            .await?
        {
            DsResponse::CheckResult(decision) => {
                //TODO! -- add metrics
                println!("Got decision: {}", decision);
                Ok(Response::new(CheckResponse {
                    decision: decision.into(),
                }))
            }
            DsResponse::Error(status) => Err(status),
            _ => Err(Status::internal("Got unexpected answer from datastore")),
        }
    }
}
