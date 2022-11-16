#![warn(missing_docs)]

//! Internal messages

use tokio::sync::oneshot::Sender;
use tonic::Status;

use crate::proto::entities::{
    AddEntityRequest, Entity, GetAllEntitiesRequest, ModifyEntityRequest, RemoveEntityRequest,
};
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
}

#[derive(Debug)]
pub enum DsResponse {
    Error(Status),

    SingleTarget(Target),
    MultipleTargets(Vec<Target>),

    SingleEntity(Entity),
    MultipleEntities(Vec<Entity>),
}
