//! The main Gatehouse server binary

use flume::Sender;
use tokio::sync::oneshot::{channel, Receiver};
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

use crate::ds::Datastore;
use crate::msgs::{DsRequest, DsResponse};
use crate::proto::base::gatehouse_server::Gatehouse;
use crate::proto::entities::{
    AddEntityRequest, EntityResponse, GetAllEntitiesRequest, ModifyEntityRequest,
    MultiEntityResponse, RemoveEntityRequest,
};
use crate::proto::groups::{
    AddGroupRequest, GetAllGroupsRequest, GroupResponse, ModifyGroupRequest, MultiGroupResponse,
    RemoveGroupRequest,
};
use crate::proto::roles::{
    AddRoleRequest, GetAllRolesRequest, MultiRoleResponse, RemoveRoleRequest, RoleResponse,
};
use crate::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, MultiTargetResponse,
    RemoveTargetRequest, TargetResponse,
};

#[derive(Debug)]
/// The core Gatehouse server
pub struct GatehouseSvc {
    dstx: Sender<DsRequest>,
}

impl GatehouseSvc {
    /// Create a new Gatehouse service
    pub async fn new() -> Self {
        let dstx = Datastore::create().await;
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
            _ = sleep(Duration::from_secs(10)) => {
                // TODO! -- add metrics
                eprintln!("Timeout waiting of target addition");
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
        request: Request<GetAllTargetsRequest>,
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

    /// Add an entity
    async fn add_entity(
        &self,
        request: Request<AddEntityRequest>,
    ) -> Result<Response<EntityResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        // wait for the datastore to respond
        match self
            .call_datastore(DsRequest::AddEntity(req.clone(), tx), "add entity", rx)
            .await?
        {
            DsResponse::SingleEntity(entity) => {
                //TODO! -- add metrics
                println!("Added entity {}", entity);
                Ok(Response::new(EntityResponse {
                    entity: Some(entity),
                }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Modify an entity
    async fn modify_entity(
        &self,
        request: Request<ModifyEntityRequest>,
    ) -> Result<Response<EntityResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::ModifyEntity(req.clone(), tx),
                "modify entity",
                rx,
            )
            .await?
        {
            DsResponse::SingleEntity(entity) => {
                //TODO! -- add metrics
                println!("Modify entity {}", entity);
                Ok(Response::new(EntityResponse {
                    entity: Some(entity),
                }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Remove an entity
    async fn remove_entity(
        &self,
        request: Request<RemoveEntityRequest>,
    ) -> Result<Response<EntityResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(
                DsRequest::RemoveEntity(req.clone(), tx),
                "remove entity",
                rx,
            )
            .await?
        {
            DsResponse::SingleEntity(entity) => {
                //TODO! -- add metrics
                println!("Remove entity {}", entity);
                Ok(Response::new(EntityResponse {
                    entity: Some(entity),
                }))
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Get all entries
    async fn get_entities(
        &self,
        request: Request<GetAllEntitiesRequest>,
    ) -> Result<Response<MultiEntityResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        match self
            .call_datastore(DsRequest::GetEntities(req.clone(), tx), "get entities", rx)
            .await?
        {
            DsResponse::MultipleEntities(entities) => {
                //TODO! -- add metrics
                println!("Got {} entities", entities.len());
                return Ok(Response::new(MultiEntityResponse { entities }));
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
        request: Request<GetAllRolesRequest>,
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
        request: Request<GetAllGroupsRequest>,
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
}
