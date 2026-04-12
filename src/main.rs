pub mod network;
pub mod handler;
pub mod protocol;
pub mod auth;
pub mod config;
pub mod error;

use crate::network::Server;
use {
    std::sync::Arc,
};

//              //
//  re-exports  //
//              //
pub use error::Error;
//              //
//              //
//              //

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = Arc::new(config::conf()?);

    let mut server = Server::new(conf);

    server.listen().await;

    Ok(())
}

