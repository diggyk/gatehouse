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
                write!(
                    f,
                    "{}/{}: {}",
                    self.typestr,
                    self.id,
                    self.actions
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

pub(crate) mod ds;
pub(crate) mod msgs;
pub(crate) mod storage;
pub mod svc;
pub(crate) mod target;
