use std::error::Error;

mod server;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = server::Server::default();
    server.start().await?;
    Ok(())
}
