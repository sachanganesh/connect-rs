mod schema;

use crate::schema::hello_world::HelloWorld;
use async_std::{io, task};
use connect::tls::rustls::internal::pemfile::{certs, rsa_private_keys};
use connect::tls::rustls::{NoClientAuth, ServerConfig};
use connect::tls::TlsServer;
use connect::{SinkExt, StreamExt};
use log::*;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Get ip address from cmd line args
    let (ip_address, cert_path, key_path) = parse_args();

    let certs = certs(&mut BufReader::new(File::open(cert_path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))?;

    let mut keys = rsa_private_keys(&mut BufReader::new(File::open(key_path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    // create a server
    let mut server = TlsServer::new(ip_address, config.into()).await?;

    // handle server connections
    // wait for a connection to come in and be accepted
    loop {
        match server.next().await {
            Some(mut conn) => {
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

            None => break,
        }
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

    let cert_path = match args.get(2) {
        Some(d) => d,
        None => {
            error!("Need to pass path to cert file as second command line argument");
            panic!();
        }
    };

    let key_path = match args.get(3) {
        Some(d) => d,
        None => {
            error!("Need to pass path to key file as third command line argument");
            panic!();
        }
    };

    (
        ip_address.to_string(),
        cert_path.to_string(),
        key_path.to_string(),
    )
}
