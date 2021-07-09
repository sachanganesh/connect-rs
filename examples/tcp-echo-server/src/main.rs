use async_std::task;
use connect::tcp::TcpListener;
use connect::{ConnectDatagram, SinkExt, StreamExt};
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
    let mut server = TcpListener::bind(ip_address).await?;

    // handle server connections
    // wait for a connection to come in and be accepted
    while let Some(mut conn) = server.next().await {
        info!("Handling connection from {}", conn.peer_addr());

        task::spawn(async move {
            while let Some(envelope) = conn.reader().next().await {
                // handle message based on intended recipient
                if envelope.tag() == 65535 {
                    // if recipient is 65535, we do custom processing
                    let data = envelope.data().to_vec();
                    let msg =
                        String::from_utf8(data).expect("could not build String from payload bytes");
                    info!("Received a message \"{}\" from {}", msg, conn.peer_addr());

                    let reply = ConnectDatagram::with_tag(envelope.tag(), msg.into_bytes())
                        .expect("could not construct new datagram from built String");

                    conn.writer()
                        .send(reply)
                        .await
                        .expect("Could not send message back to source connection");
                    info!("Sent message back to original sender");
                } else {
                    // if recipient is anything else, we just send it back
                    warn!(
                        "Received a message for unknown recipient {} from {}",
                        envelope.tag(),
                        conn.peer_addr()
                    );

                    conn.writer()
                        .send(envelope)
                        .await
                        .expect("Could not send message back to source connection");
                    info!("Sent message back to original sender");
                }
            }
        });
    }

    Ok(())
}
