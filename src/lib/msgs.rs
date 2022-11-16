#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::targets::{
    AddTargetRequest, GetAllTargetsRequest, ModifyTargetRequest, RemoveTargetRequest, Target,
};

#[derive(Debug)]
pub(crate) enum DsRequest {
    AddTarget(AddTargetRequest, Sender<DsResponse>),
    ModifyTarget(ModifyTargetRequest, Sender<DsResponse>),
    RemoveTarget(RemoveTargetRequest, Sender<DsResponse>),
    GetTargets(GetAllTargetsRequest, Sender<DsResponse>),
}

#[derive(Debug)]
pub enum DsResponse {
    SingleTarget(Target),
    MultipleTargets(Vec<Target>),
    Error(Status),
}
