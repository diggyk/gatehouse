#![warn(missing_docs)]

//! Gatehouse libraries

/// Gatehouse protobuf definitions
pub mod proto {
    /// Base protobufs for the server and client
    pub mod base {
        tonic::include_proto!("gatehouse");
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
                    "{}/{}: {} // {}",
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

    /// Common protobufs between other packages
    pub mod common {
        tonic::include_proto!("common");
    }
}

pub(crate) mod ds;
pub(crate) mod msgs;
pub(crate) mod storage;
pub mod svc;
pub(crate) mod target;
