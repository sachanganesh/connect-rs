use connect::tls::rustls::ClientConfig;
use connect::{ConnectDatagram, Connection, SinkExt, StreamExt};
use log::*;
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
    let mut conn = Connection::tls_client(ip_addr, &domain, client_config.into()).await?;

    // send a message to the server
    let msg = String::from("Hello world");
    info!("Sending message: {}", msg);

    let envelope = ConnectDatagram::new(65535, msg.into_bytes())?;
    conn.writer().send(envelope).await?;

    // wait for the server to reply with an ack
    if let Some(mut reply) = conn.reader().next().await {
        let data = reply.take_data().unwrap();
        let msg = String::from_utf8(data)?;

        info!("Received message: {}", msg);
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
