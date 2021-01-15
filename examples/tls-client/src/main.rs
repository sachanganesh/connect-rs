mod schema;

use crate::schema::hello_world::HelloWorld;
use connect::tls::rustls::ClientConfig;
use connect::{Connection, SinkExt, StreamExt};
use log::*;
use protobuf::well_known_types::Any;
use std::env;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // get ip address and domain from cmd line args
    let (ip_addr, domain, cafile_path) = parse_args();

    // construct `rustls` client config
    let cafile = std::fs::read(cafile_path)?;

    let mut client_pem = std::io::Cursor::new(cafile);

    let mut client_config = ClientConfig::new();
    client_config
        .root_store
        .add_pem_file(&mut client_pem)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cert"))?;

    // create a client connection to the server
    let mut conn = Connection::tls_client(ip_addr, &domain, client_config.into())?;

    // send a message to the server
    let raw_msg = String::from("Hello world");
    info!("Sending message: {}", raw_msg);

    let mut msg = HelloWorld::new();
    msg.set_message(raw_msg);

    conn.writer().send(msg).await?;

    // wait for the server to reply with an ack
    while let Some(reply) = conn.reader().next().await {
        info!("Received message");

        let msg: HelloWorld = Any::unpack(&reply)?.unwrap();

        info!("Unpacked reply: {}", msg.get_message());
    }

    Ok(())
}

fn parse_args() -> (String, String, String) {
    let args: Vec<String> = env::args().collect();

    let ip_address = match args.get(1) {
        Some(addr) => addr,
        None => {
            error!("Need to pass IP address to connect to as first command line argument");
            panic!();
        }
    };

    let domain = match args.get(2) {
        Some(d) => d,
        None => {
            error!("Need to pass domain name as second command line argument");
            panic!();
        }
    };

    let cafile_path = match args.get(3) {
        Some(d) => d,
        None => {
            error!("Need to pass path to cafile as third command line argument");
            panic!();
        }
    };

    (
        ip_address.to_string(),
        domain.to_string(),
        cafile_path.to_string(),
    )
}
