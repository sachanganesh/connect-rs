mod schema;

use crate::schema::hello_world::HelloWorld;
use async_std::task;
use connect::tcp::TcpServer;
use connect::{SinkExt, StreamExt};
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
            error!("Need to pass IP address to bind to as command line argument");
            panic!();
        }
    };

    // create a server
    let mut server = TcpServer::new(ip_address).await?;

    // handle server connections
    // wait for a connection to come in and be accepted
    while let Some(mut conn) = server.next().await {
        info!("Handling connection from {}", conn.peer_addr());

        task::spawn(async move {
            while let Some(msg) = conn.reader().next().await {
                if msg.is::<HelloWorld>() {
                    if let Ok(Some(contents)) = msg.unpack::<HelloWorld>() {
                        info!(
                            "Received a message \"{}\" from {}",
                            contents.get_message(),
                            conn.peer_addr()
                        );

                        conn.writer()
                            .send(contents)
                            .await
                            .expect("Could not send message back to source connection");
                        info!("Sent message back to original sender");
                    }
                } else {
                    error!("Received a message of unknown type")
                }
            }
        });
    }

    Ok(())
}
