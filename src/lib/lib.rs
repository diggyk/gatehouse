#![warn(missing_docs)]

//! Gatehouse libraries

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

pub(crate) mod ds;
pub(crate) mod entity;
pub(crate) mod msgs;
pub(crate) mod storage;
pub mod svc;
pub(crate) mod target;
