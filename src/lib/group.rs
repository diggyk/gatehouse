#![warn(missing_docs)]

//! The Target type and methods
use core::hash::Hash;

use std::collections::HashSet;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::entity::RegisteredEntity;
use crate::proto::groups::{Group, GroupMember};

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct RegisteredGroupMember {
    pub name: String,
    pub typestr: String,
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredGroup {
    pub name: String,
    pub desc: Option<String>,
    pub members: HashSet<RegisteredGroupMember>,
    pub roles: HashSet<String>,
}

impl RegisteredGroup {
    pub(crate) fn new(
        name: &str,
        desc: Option<String>,
        members: HashSet<RegisteredGroupMember>,
        roles: HashSet<String>,
    ) -> Self {
        let name = name.to_ascii_lowercase();
        Self {
            name,
            desc,
            members,
            roles,
        }
    }
}

impl Display for RegisteredGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "group[{}]: {} members  {} roles",
            self.name,
            self.members.len(),
            self.roles.len()
        )
    }
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

impl From<RegisteredGroupMember> for GroupMember {
    fn from(g: RegisteredGroupMember) -> Self {
        Self {
            name: g.name,
            typestr: g.typestr,
        }
    }
}

impl From<GroupMember> for RegisteredGroupMember {
    fn from(g: GroupMember) -> Self {
        Self {
            name: g.name,
            typestr: g.typestr,
        }
    }
}
impl From<&RegisteredEntity> for RegisteredGroupMember {
    fn from(re: &RegisteredEntity) -> Self {
        Self {
            name: re.name.clone(),
            typestr: re.typestr.clone(),
        }
    }
}

impl From<RegisteredGroup> for Group {
    fn from(g: RegisteredGroup) -> Self {
        Self {
            name: g.name,
            desc: g.desc,
            members: g.members.iter().map(|m| m.clone().into()).collect(),
            roles: g.roles.into_iter().collect(),
        }
    }
}
