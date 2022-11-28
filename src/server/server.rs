#![warn(missing_docs)]

//! The main Gatehouse server binary

use tonic::transport::Server;

use gatehouse::proto::base::gatehouse_server::GatehouseServer;
use gatehouse::svc::GatehouseSvc;

#[tokio::main]
/// Our main function for the server
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:6174".parse()?;
    let svc = GatehouseSvc::new().await;

    println!("Starting Gatehouse server: {:?}", addr);
    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(GatehouseServer::new(svc)))
        .serve(addr)
        .await?;

    Ok(())
}
