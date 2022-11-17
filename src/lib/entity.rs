#![warn(missing_docs)]

//! The Entity type and methods

use core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::proto::common::AttributeValues;
use crate::proto::entities::Entity;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredEntity {
    pub name: String,
    pub typestr: String,
    pub attributes: HashMap<String, HashSet<String>>,
}

/// Two registered entities are equivalent if the name and typestr are identical
impl PartialEq for RegisteredEntity {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.typestr == other.typestr
    }
}

impl Hash for RegisteredEntity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.typestr.hash(state);
    }
}

impl From<Entity> for RegisteredEntity {
    fn from(tgt: Entity) -> Self {
        let mut attributes = HashMap::new();
        for kv in tgt.attributes {
            attributes.insert(kv.0, HashSet::from_iter(kv.1.values));
        }

        Self {
            name: tgt.name.to_ascii_lowercase(),
            typestr: tgt.typestr.to_ascii_lowercase(),
            attributes,
        }
    }
}

impl From<RegisteredEntity> for Entity {
    fn from(target: RegisteredEntity) -> Self {
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
            attributes,
        }
    }
}

impl Display for RegisteredEntity {
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
        write!(f, "ent[{}/{}]: {}", self.typestr, self.name, attribvals)
    }
}

impl RegisteredEntity {
    pub(crate) fn new(
        name: &str,
        typestr: &str,
        attributes: HashMap<String, HashSet<String>>,
    ) -> Self {
        RegisteredEntity {
            name: name.to_string(),
            typestr: typestr.to_string(),
            attributes,
        }
    }
}
