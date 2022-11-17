use core::hash::Hash;
use std::collections::HashSet;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::proto::roles::Role;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredRole {
    pub name: String,
    pub groups: HashSet<String>,
}

impl RegisteredRole {
    pub(crate) fn new(name: &str) -> Self {
        let name = name.to_ascii_lowercase();
        Self {
            name,
            groups: HashSet::new(),
        }
    }
}

impl PartialEq for RegisteredRole {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for RegisteredRole {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for RegisteredRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "role[{}] (in {} groups)", self.name, self.groups.len())
    }
}

impl From<Role> for RegisteredRole {
    fn from(role: Role) -> Self {
        Self {
            name: role.name,
            groups: HashSet::new(),
        }
    }
}

impl From<RegisteredRole> for Role {
    fn from(role: RegisteredRole) -> Self {
        Self { name: role.name }
    }
}
