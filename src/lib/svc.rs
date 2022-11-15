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

        if let Err(err) = self
            .dstx
            .send_async(DsRequest::AddTarget(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Add target failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

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

        if let Err(err) = self
            .dstx
            .send_async(DsRequest::AddTargetActions(req.clone(), tx))
            .await
        {
            // TODO! -- add metrics
            eprintln!("Add target actions failed: {:?}", err);
            return Err(Status::internal(err.to_string()));
        }

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
                    }
                }
            }
        }
    }
}
