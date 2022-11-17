#![warn(missing_docs)]

//! The Target type and methods
use core::hash::Hash;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::entity::RegisteredEntity;
use crate::proto::common::AttributeValues;
use crate::proto::targets::Target;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredGroup {
    pub name: String,
    pub members: HashSet<RegisteredEntity>,
    pub roles: HashSet<String>,
}

impl PartialEq for RegisteredGroup {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for RegisteredGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
