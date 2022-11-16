#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::targets::{
    AddTargetActionRequest, AddTargetRequest, GetAllTargetsRequest, RemoveTargetActionRequest,
    RemoveTargetRequest, Target,
};

#[derive(Debug)]
pub(crate) enum DsRequest {
    AddTarget(AddTargetRequest, Sender<DsResponse>),
    AddTargetActions(AddTargetActionRequest, Sender<DsResponse>),
    RemoveTargetActions(RemoveTargetActionRequest, Sender<DsResponse>),
    RemoveTarget(RemoveTargetRequest, Sender<DsResponse>),
    GetTargets(GetAllTargetsRequest, Sender<DsResponse>),
}

#[derive(Debug)]
pub enum DsResponse {
    SingleTarget(Target),
    MultipleTargets(Vec<Target>),
    Error(Status),
}
