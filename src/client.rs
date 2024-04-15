use zbus::{zvariant::ObjectPath, proxy, Connection, Result};
use futures_util::stream::StreamExt;

#[proxy(
    interface = "org.zbus.mplayer",
    default_service = "org.zbus.mplayer",
    default_path = "/org/zbus/mplayer"
)]
trait SystemdManager {
    #[zbus(property)]
    fn architecture(&self) -> Result<String>;
    #[zbus(property)]
    fn environment(&self) -> Result<Vec<String>>;
}

#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::system().await?;

    let proxy = SystemdManagerProxy::new(&connection).await?;
    println!("Host architecture: {}", proxy.architecture().await?);
    println!("Environment:");
    for env in proxy.environment().await? {
        println!("  {}", env);
    }

    Ok(())
}
