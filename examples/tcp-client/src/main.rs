pub mod schema;

use crate::schema::hello_world::HelloWorld;
use connect::{Connection, SinkExt, StreamExt};
use log::*;
use protobuf::well_known_types::Any;
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
    let raw_msg = String::from("Hello world");

    let mut msg = HelloWorld::new();
    msg.set_message(raw_msg.clone());

    conn.writer().send(msg).await?;
    info!("Sent message: {}", raw_msg);

    // wait for the server to reply with an ack
    while let Some(reply) = conn.reader().next().await {
        info!("Received message");

        let msg: HelloWorld = Any::unpack(&reply)?.unwrap();

        info!("Unpacked reply: {}", msg.get_message());
    }

    Ok(())
}
