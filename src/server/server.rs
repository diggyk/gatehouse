#![warn(missing_docs)]

//! The main Gatehouse server binary

use std::process::exit;

use tonic::transport::Server;

use gatehouse::helpers::str;
use gatehouse::proto::base::gatehouse_server::GatehouseServer;
use gatehouse::svc::GatehouseSvc;
use gatehouse::StorageType;

fn parse_storage_param(val: &str) -> StorageType {
    if let Some((typestr, val)) = val.split_once(':') {
        match typestr.to_ascii_lowercase().as_str() {
            "file" => return StorageType::FileSystem(str(val)),
            "nil" => return StorageType::Nil,
            _ => {
                eprintln!("Unknown storage type: {}", typestr.to_ascii_lowercase());
                exit(1);
            }
        }
    }

    StorageType::FileSystem(str("/tmp/gatehouse"))
}

#[tokio::main]
/// Our main function for the server
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:6174".parse()?;
    let storage = parse_storage_param(
        std::env::var("GATESTORAGE")
            .unwrap_or_else(|_| str("file:/tmp/gatehouse"))
            .as_str(),
    );

    let svc = GatehouseSvc::new(&storage).await;

    println!("Starting Gatehouse server:");
    println!("\x7f addr: {}", addr);
    println!("\x7f storage: {}", storage);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(GatehouseServer::new(svc)))
        .serve(addr)
        .await?;

    Ok(())
}
