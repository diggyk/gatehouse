#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::targets::{AddTargetActionRequest, AddTargetRequest, Target};

#[derive(Debug)]
pub(crate) enum DsRequest {
    AddTarget(AddTargetRequest, Sender<DsResponse>),
    AddTargetActions(AddTargetActionRequest, Sender<DsResponse>),
}

#[derive(Debug)]
pub enum DsResponse {
    SingleTarget(Target),
    Error(Status),
}
