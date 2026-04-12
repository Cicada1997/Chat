pub mod network;
pub mod handler;
pub mod protocol;
pub mod auth;
pub mod error;

//              //
//  re-exports  //
//              //
pub use error::Error;

use network::Server;

#[tokio::main]
async fn main() -> Result<(), Error> {

    let mut server = Server::new("127.0.0.1:5225");

    server.listen().await;

    Ok(())
}

