#![warn(missing_docs)]

//! The Target type and methods

use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::fmt::Display;

use crate::proto::targets::Target;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RegisteredTarget {
    pub name: String,
    pub typestr: String,
    pub actions: HashSet<String>,
}

impl From<Target> for RegisteredTarget {
    fn from(tgt: Target) -> Self {
        let mut actions = HashSet::new();

        for action in tgt.actions {
            actions.insert(action.to_ascii_lowercase());
        }

        Self {
            name: tgt.id.to_ascii_lowercase(),
            typestr: tgt.typestr.to_ascii_lowercase(),
            actions,
        }
    }
}

impl From<RegisteredTarget> for Target {
    fn from(target: RegisteredTarget) -> Self {
        Self {
            id: target.name,
            typestr: target.typestr,
            actions: target.actions.iter().map(|a| a.to_string()).collect(),
        }
    }
}

impl Display for RegisteredTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}: {}",
            self.typestr,
            self.name,
            self.actions
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl RegisteredTarget {
    pub(crate) fn new(name: &str, typestr: &str, actions: Vec<String>) -> Self {
        let mut actions_set = HashSet::new();

        for action in actions {
            actions_set.insert(action);
        }

        RegisteredTarget {
            name: name.to_string(),
            typestr: typestr.to_string(),
            actions: actions_set,
        }
    }
}
