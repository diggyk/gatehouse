#![warn(missing_docs)]

//! The Target type and methods

use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::proto::common::AttributeValues;
use crate::proto::targets::Target;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RegisteredTarget {
    pub name: String,
    pub typestr: String,
    pub actions: HashSet<String>,
    pub attributes: HashMap<String, HashSet<String>>,
}

impl From<Target> for RegisteredTarget {
    fn from(tgt: Target) -> Self {
        let mut actions = HashSet::new();
        for action in tgt.actions {
            actions.insert(action.to_ascii_lowercase());
        }

        let mut attributes = HashMap::new();
        for kv in tgt.attributes {
            attributes.insert(kv.0, HashSet::from_iter(kv.1.values));
        }

        Self {
            name: tgt.name.to_ascii_lowercase(),
            typestr: tgt.typestr.to_ascii_lowercase(),
            actions,
            attributes,
        }
    }
}

impl From<RegisteredTarget> for Target {
    fn from(target: RegisteredTarget) -> Self {
        let mut attributes = HashMap::new();
        for kv in target.attributes {
            attributes.insert(
                kv.0,
                AttributeValues {
                    values: kv.1.iter().map(|v| v.to_string()).collect(),
                },
            );
        }

        Self {
            name: target.name,
            typestr: target.typestr,
            actions: target.actions.iter().map(|a| a.to_string()).collect(),
            attributes,
        }
    }
}

impl Display for RegisteredTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attribvals = self
            .attributes
            .iter()
            .map(|kv| {
                format!(
                    " {}: {}",
                    kv.0,
                    kv.1.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>()
            .join(" ");
        write!(
            f,
            "{}/{}: {} // {}",
            self.typestr,
            self.name,
            self.actions
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(","),
            attribvals
        )
    }
}

impl RegisteredTarget {
    pub(crate) fn new(
        name: &str,
        typestr: &str,
        actions: Vec<String>,
        attributes: HashMap<String, HashSet<String>>,
    ) -> Self {
        let mut actions_set = HashSet::new();

        for action in actions {
            actions_set.insert(action);
        }

        RegisteredTarget {
            name: name.to_string(),
            typestr: typestr.to_string(),
            actions: actions_set,
            attributes,
        }
    }
}
