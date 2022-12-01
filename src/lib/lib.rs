#![warn(missing_docs)]

//! Gatehouse libraries

use std::fmt::Display;

/// Gatehouse protobuf definitions
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod proto {
    /// Common protobufs between other packages
    pub mod common {
        tonic::include_proto!("common");
    }

    /// Base protobufs for the server and client
    pub mod base {
        tonic::include_proto!("gatehouse");
    }

    /// Actor related protobufs
    pub mod actors {
        use std::fmt::Display;

        tonic::include_proto!("actors");

        impl Display for Actor {
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
    /// indicates an Etcd backend should be used with the given url
    Etcd(String),
}
impl Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::FileSystem(path) => write!(f, "File system({})", path),
            Self::Etcd(url) => write!(f, "Etcd({})", url),
        }
    }
}
impl From<&str> for StorageType {
    fn from(val: &str) -> Self {
        Self::new(val)
    }
}
impl From<String> for StorageType {
    fn from(val: String) -> Self {
        Self::new(&val)
    }
}
impl StorageType {
    fn new(val: &str) -> Self {
        if let Some((typestr, val)) = val.split_once(':') {
            match typestr.to_ascii_lowercase().as_str() {
                "etcd" => return Self::Etcd(val.to_string()),
                "file" => return Self::FileSystem(val.to_string()),
                "nil" => return Self::Nil,
                _ => {
                    eprintln!("Unknown storage type: {}", typestr.to_ascii_lowercase());
                    std::process::exit(1);
                }
            }
        }

        Self::FileSystem("/tmp/gatehouse".to_string())
    }
}

pub(crate) mod actor;
pub(crate) mod ds;
pub(crate) mod group;
pub mod helpers;
pub(crate) mod msgs;
pub(crate) mod policy;
pub(crate) mod role;
pub(crate) mod storage;
pub mod svc;
pub(crate) mod target;
