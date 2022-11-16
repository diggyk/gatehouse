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
    /// Send a request to the datastore
    async fn send_ds_request(&self, req: DsRequest, op: &str) -> Result<(), Status> {
        if let Err(err) = self.dstx.send_async(req).await {
            // TODO! -- add metrics
            eprintln!("{} failed: {:?}", op, err);
            return Err(Status::internal(err.to_string()));
        }
        Ok(())
    }

    /// Wait for a response from the datastore
    async fn wait_for_response(&self, rx: Receiver<DsResponse>) -> Result<DsResponse, Status> {
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
    /// Add a new target
    async fn add_target(
        &self,
        request: Request<AddTargetRequest>,
    ) -> Result<Response<TargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        // Ask the datastore to add this target
        self.send_ds_request(DsRequest::AddTarget(req.clone(), tx), "add target")
            .await?;

        // Wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // Ask the datastore to update the target
        self.send_ds_request(DsRequest::ModifyTarget(req.clone(), tx), "modify target")
            .await?;

        // Wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // ask the datastore to process
        self.send_ds_request(DsRequest::RemoveTarget(req.clone(), tx), "remove target")
            .await?;

        // Wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // ask the datastore to process
        self.send_ds_request(DsRequest::GetTargets(req.clone(), tx), "get target(s)")
            .await?;

        // Wait for the datastore to respond
        match self.wait_for_response(rx).await? {
            DsResponse::MultipleTargets(tgts) => {
                //TODO! -- add metrics
                println!("Got {} targets", tgts.len());
                return Ok(Response::new(MultiTargetResponse { targets: tgts }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }

    /// Add an entity
    async fn add_entity(
        &self,
        request: Request<AddEntityRequest>,
    ) -> Result<Response<EntityResponse>, Status> {
        let req = request.into_inner();
        let (tx, rx) = channel::<DsResponse>();

        // ask the datastore to process
        self.send_ds_request(DsRequest::AddEntity(req.clone(), tx), "add entity")
            .await?;

        // wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // ask the datastore to process
        self.send_ds_request(DsRequest::ModifyEntity(req.clone(), tx), "modify entity")
            .await?;

        // wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // ask the datastore to process
        self.send_ds_request(DsRequest::RemoveEntity(req.clone(), tx), "remove entity")
            .await?;

        // wait for the datastore to respond
        match self.wait_for_response(rx).await? {
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

        // ask the datastore to process
        self.send_ds_request(DsRequest::GetEntities(req.clone(), tx), "get entities")
            .await?;

        // Wait for the datastore to respond
        match self.wait_for_response(rx).await? {
            DsResponse::MultipleEntities(entities) => {
                //TODO! -- add metrics
                println!("Got {} entities", entities.len());
                return Ok(Response::new(MultiEntityResponse { entities }));
            }
            DsResponse::Error(status) => return Err(status),
            _ => return Err(Status::internal("Got unexpected answer from datastore")),
        }
    }
}
