use clap::Parser;
use mini_redis::{server, DEFAULT_PORT};
use tokio::{
    net::TcpListener,
    signal::{self},
};

#[tokio::main]
pub async fn main() -> mini_redis::Result<()> {
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);

    // Bind a TCP Listener
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
    port: Option<u16>,
}
