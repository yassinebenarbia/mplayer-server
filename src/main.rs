use std::error::Error;

mod server;

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = server::Server::default();
    server.start().await?;
    Ok(())
}
