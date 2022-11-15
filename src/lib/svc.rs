//! The main Gatehouse server binary

use flume::Sender;
use tokio::sync::oneshot::channel;
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

use crate::ds::Datastore;
use crate::msgs::{DsRequest, DsResponse};
use crate::proto::base::gatehouse_server::Gatehouse;
use crate::proto::targets::{
    AddTargetActionRequest, AddTargetActionResponse, AddTargetRequest, AddTargetResponse,
    GetAllTargetsRequest, GetAllTargetsResponse, RemoveTargetActionRequest,
    RemoveTargetActionResponse, RemoveTargetRequest, RemoveTargetResponse,
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

#[tonic::async_trait]
impl Gatehouse for GatehouseSvc {
    /// Add a new target
    async fn add_target(
        &self,
        request: Request<AddTargetRequest>,
    ) -> Result<Response<AddTargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, mut rx) = channel::<DsResponse>();

        // Ask the datastore to add this target
        if let Err(err) = self
            .dstx
            .send_async(DsRequest::AddTarget(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Add target failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

        // Wait for the datastore to respond
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {
                    // TODO! -- add metrics
                    eprintln!("Timeout waiting of target addition");
                    return Err(Status::deadline_exceeded("Timeout waiting for response from datastore"));
                },
                msg = &mut rx => {
                    match msg.unwrap() {
                        DsResponse::SingleTarget(tgt) => {
                            //TODO! -- add metrics
                            println!("Added target {}", tgt);
                            return Ok(Response::new(AddTargetResponse { target: Some(tgt) }));
                        }
                        DsResponse::Error(status) => return Err(status),
                        _ => return Err(Status::internal("Got unexpected answer from datastore"))
                    }

                }
            }
        }
    }

    /// Add new actions to a target
    async fn add_target_action(
        &self,
        request: Request<AddTargetActionRequest>,
    ) -> Result<Response<AddTargetActionResponse>, Status> {
        let req = request.into_inner();
        let (tx, mut rx) = channel::<DsResponse>();

        // Ask the datastore to update the target actions
        if let Err(err) = self
            .dstx
            .send_async(DsRequest::AddTargetActions(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Add target actions failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

        // Wait for the datastore to respond
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {
                    // TODO! -- add metrics
                    eprintln!("Timeout waiting of target action addition");
                    return Err(Status::deadline_exceeded("Timeout waiting for response from datastore"));
                },
                msg = &mut rx => {
                    match msg.unwrap() {
                        DsResponse::SingleTarget(tgt) => {
                            //TODO! -- add metrics
                            println!("Added target actions: {}", tgt);
                            return Ok(Response::new(AddTargetActionResponse { target: Some(tgt) }));
                        }
                        DsResponse::Error(status) => return Err(status),
                        _ => return Err(Status::internal("Got unexpected answer from datastore"))
                    }
                }
            }
        }
    }

    /// Remove an action from a target
    async fn remove_target_action(
        &self,
        request: Request<RemoveTargetActionRequest>,
    ) -> Result<Response<RemoveTargetActionResponse>, Status> {
        let req = request.into_inner();
        let (tx, mut rx) = channel::<DsResponse>();

        // ask the datastore to process
        if let Err(err) = self
            .dstx
            .send_async(DsRequest::RemoveTargetActions(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Remove target action failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

        // Wait for the datastore to respond
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {
                    // TODO! -- add metrics
                    eprintln!("Timeout waiting of target addition");
                    return Err(Status::deadline_exceeded("Timeout waiting for response from datastore"));
                },
                msg = &mut rx => {
                    match msg.unwrap() {
                        DsResponse::SingleTarget(tgt) => {
                            //TODO! -- add metrics
                            println!("Removed action(s) from target {}", tgt);
                            return Ok(Response::new(RemoveTargetActionResponse{ target: Some(tgt) }));
                        }
                        DsResponse::Error(status) => return Err(status),
                        _ => return Err(Status::internal("Got unexpected answer from datastore"))
                    }

                }
            }
        }
    }

    /// Remove an existing target
    async fn remove_target(
        &self,
        request: Request<RemoveTargetRequest>,
    ) -> Result<Response<RemoveTargetResponse>, Status> {
        let req = request.into_inner();
        let (tx, mut rx) = channel::<DsResponse>();

        // ask the datastore to process
        if let Err(err) = self
            .dstx
            .send_async(DsRequest::RemoveTarget(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Remove target failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

        // Wait for the datastore to respond
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {
                    // TODO! -- add metrics
                    eprintln!("Timeout waiting of target addition");
                    return Err(Status::deadline_exceeded("Timeout waiting for response from datastore"));
                },
                msg = &mut rx => {
                    match msg.unwrap() {
                        DsResponse::SingleTarget(tgt) => {
                            //TODO! -- add metrics
                            println!("Removed target {}", tgt);
                            return Ok(Response::new(RemoveTargetResponse { target: Some(tgt) }));
                        }
                        DsResponse::Error(status) => return Err(status),
                        _ => return Err(Status::internal("Got unexpected answer from datastore"))
                    }

                }
            }
        }
    }

    /// Get all targets
    async fn get_targets(
        &self,
        request: Request<GetAllTargetsRequest>,
    ) -> Result<Response<GetAllTargetsResponse>, Status> {
        let req = request.into_inner();
        let (tx, mut rx) = channel::<DsResponse>();

        // ask the datastore to process
        if let Err(err) = self
            .dstx
            .send_async(DsRequest::GetTargets(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Get targets failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

        // Wait for the datastore to respond
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {
                    // TODO! -- add metrics
                    eprintln!("Timeout waiting of targets list");
                    return Err(Status::deadline_exceeded("Timeout waiting for response from datastore"));
                },
                msg = &mut rx => {
                    match msg.unwrap() {
                        DsResponse::MultipleTargets(tgts) => {
                            //TODO! -- add metrics
                            println!("Got {} targets", tgts.len());
                            return Ok(Response::new(GetAllTargetsResponse { targets: tgts }));
                        }
                        DsResponse::Error(status) => return Err(status),
                        _ => return Err(Status::internal("Got unexpected answer from datastore"))
                    }

                }
            }
        }
    }
}
