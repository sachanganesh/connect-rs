use async_std::{io, task};
use connect::{tls::TlsListener, ConnectDatagram, SinkExt, StreamExt};
use log::*;
use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, rsa_private_keys};
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
    assert!(certs.len() > 0);

    let rustls_certs: Vec<Certificate> = certs.into_iter().map(|v| Certificate(v)).collect();
    assert!(rustls_certs.len() > 0);

    let mut keys = rsa_private_keys(&mut BufReader::new(File::open(key_path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(rustls_certs, PrivateKey(keys.remove(0)))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    // create a server
    let mut server = TlsListener::bind(ip_address, config.into()).await?;

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
