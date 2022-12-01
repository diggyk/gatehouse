#![warn(missing_docs)]

//! The Actor type and methods

use core::hash::Hash;
use fasthash::metro;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::proto::actors::Actor;
use crate::proto::common::AttributeValues;
use crate::proto::groups::GroupMember;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredActor {
    pub name: String,
    pub typestr: String,
    pub attributes: HashMap<String, HashSet<String>>,
}

/// Two registered actors are equivalent if the name and typestr are identical
impl PartialEq for RegisteredActor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.typestr == other.typestr
    }
}

impl Hash for RegisteredActor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.typestr.hash(state);
    }
}

impl From<Actor> for RegisteredActor {
    fn from(tgt: Actor) -> Self {
        let mut attributes = HashMap::new();
        for (key, val) in tgt.attributes {
            attributes.insert(key, HashSet::from_iter(val.values));
        }

        Self {
            name: tgt.name.to_ascii_lowercase(),
            typestr: tgt.typestr.to_ascii_lowercase(),
            attributes,
        }
    }
}

impl From<RegisteredActor> for Actor {
    fn from(actor: RegisteredActor) -> Self {
        let mut attributes = HashMap::new();
        for (key, val) in actor.attributes {
            attributes.insert(
                key,
                AttributeValues {
                    values: val.iter().map(|v| v.to_string()).collect(),
                },
            );
        }

        Self {
            name: actor.name,
            typestr: actor.typestr,
            attributes,
        }
    }
}

impl From<RegisteredActor> for GroupMember {
    fn from(actor: RegisteredActor) -> Self {
        Self {
            name: actor.name,
            typestr: actor.typestr,
        }
    }
}

impl Display for RegisteredActor {
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

impl RegisteredActor {
    pub(crate) fn new(
        name: &str,
        typestr: &str,
        attributes: HashMap<String, HashSet<String>>,
    ) -> Self {
        RegisteredActor {
            name: name.to_string(),
            typestr: typestr.to_string(),
            attributes,
        }
    }

    /// calculate the bucket for this entry
    pub(crate) fn bucket(&self) -> u8 {
        let hash = metro::hash64(format!("{}/{}", self.typestr, self.name));
        (hash % 100).try_into().unwrap()
    }
}
