#![warn(missing_docs)]

//! Gatehouse libraries

use std::fmt::Display;

/// Gatehouse protobuf definitions
pub mod proto {
    /// Common protobufs between other packages
    pub mod common {
        tonic::include_proto!("common");
    }

    /// Base protobufs for the server and client
    pub mod base {
        tonic::include_proto!("gatehouse");
    }

    /// Entity related protobufs
    pub mod entities {
        use std::fmt::Display;

        tonic::include_proto!("entities");

        impl Display for Entity {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let attribvals = self
                    .attributes
                    .iter()
                    .map(|kv| format!(" {}: {}", kv.0, kv.1.values.join(", ")))
                    .collect::<Vec<String>>()
                    .join(" ");

                write!(f, "ent[{}/{}]: {}", self.typestr, self.name, attribvals)
            }
        }
    }

    /// Group related protobufs
    pub mod groups {
        use std::fmt::Display;

        tonic::include_proto!("groups");

        impl Display for Group {
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
    }

    /// Protobufs for policies
    pub mod policies {
        use std::fmt::Display;

        tonic::include_proto!("policies");

        impl Display for PolicyRule {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "policy[{}]", self.name)
            }
        }
    }

    /// Role related protobufs
    pub mod roles {
        use std::fmt::Display;

        tonic::include_proto!("roles");

        impl Display for Role {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "role[{}]", self.name)
            }
        }
    }

    /// Protobufs related to target operations
    pub mod targets {
        use std::fmt::Display;

        tonic::include_proto!("targets");

        impl Display for Target {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let attribvals = self
                    .attributes
                    .iter()
                    .map(|kv| format!(" {}: {}", kv.0, kv.1.values.join(", ")))
                    .collect::<Vec<String>>()
                    .join(" ");

                write!(
                    f,
                    "tgt[{}/{}]: {} // {}",
                    self.typestr,
                    self.name,
                    self.actions
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    attribvals
                )
            }
        }
    }
}

/// Specify the type of persistent backend to use
pub enum StorageType {
    /// indicates no backend should be used, useful for unit tests
    Nil,
    /// indicates a file backend should be used at the given path
    FileSystem(String),
}
impl Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::FileSystem(path) => write!(f, "File system: Path {}", path),
        }
    }
}

pub(crate) mod ds;
pub(crate) mod entity;
pub(crate) mod group;
pub mod helpers;
pub(crate) mod msgs;
pub(crate) mod policy;
pub(crate) mod role;
pub(crate) mod storage;
pub mod svc;
pub(crate) mod target;
