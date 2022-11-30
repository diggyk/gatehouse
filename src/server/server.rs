#![warn(missing_docs)]

//! The main Gatehouse server binary

use tonic::transport::Server;

use gatehouse::helpers::str;
use gatehouse::proto::base::gatehouse_server::GatehouseServer;
use gatehouse::svc::GatehouseSvc;

#[tokio::main]
/// Our main function for the server
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:6174".parse()?;
    let storage = std::env::var("GATESTORAGE")
        .unwrap_or_else(|_| str("file:/tmp/gatehouse"))
        .into();

    let svc = GatehouseSvc::new(&storage).await;

    println!("Starting Gatehouse server:");
    println!("* addr: {}", addr);
    println!("* storage: {}", storage);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(GatehouseServer::new(svc)))
        .serve(addr)
        .await?;

    Ok(())
}
