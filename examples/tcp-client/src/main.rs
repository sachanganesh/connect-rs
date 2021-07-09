use connect::{ConnectDatagram, Connection, SinkExt, StreamExt};
use log::*;
use std::env;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Get ip address from cmd line args
    let args: Vec<String> = env::args().collect();
    let ip_address = match args.get(1) {
        Some(addr) => addr,
        None => {
            error!("Need to pass IP address to connect to as command line argument");
            panic!();
        }
    };

    // create a client connection to the server
    let mut conn = Connection::tcp_client(ip_address).await?;

    // send a message to the server
    let msg = String::from("Hello world");
    info!("Sending message: {}", msg);

    let envelope = ConnectDatagram::with_tag(65535, msg.into_bytes())?;
    conn.writer().send(envelope).await?;

    // wait for the server to reply with an ack
    if let Some(reply) = conn.reader().next().await {
        let data = reply.data().to_vec();
        let msg = String::from_utf8(data)?;

        info!("Received message: {}", msg);
    }

    Ok(())
}
