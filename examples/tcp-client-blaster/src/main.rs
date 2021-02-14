use connect::{ConnectDatagram, Connection, SinkExt, StreamExt};
use log::*;
use std::env;

type Number = u16;
const NUM_MESSAGES: Number = 10000;

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
    let conn = Connection::tcp_client(ip_address).await?;
    let (mut reader, mut writer) = conn.split();

    // receive messages
    let read_task = async_std::task::spawn(async move {
        let mut prev: Option<Number> = None;

        while let Some(mut reply) = reader.next().await {
            let mut payload = reply.take_data().unwrap();

            let mut data_bytes: [u8; 2] = [0; 2];
            for i in 0..payload.len() {
                data_bytes[i] = payload.remove(0);
            }

            let data = Number::from_be_bytes(data_bytes);

            if let Some(prev_num) = prev {
                assert_eq!(prev_num + 1, data);
            } else {
                assert_eq!(0, data);
            }

            prev = Some(data);
            info!("Received message: {}", data);

            if prev.unwrap() + 1 == NUM_MESSAGES {
                break;
            }
        }
    });

    // send messages to the server
    for i in 0..NUM_MESSAGES {
        info!("Sending message: {}", i);
        let data = i.to_be_bytes().to_vec();
        let envelope = ConnectDatagram::new(i, data)?;
        writer.send(envelope).await?;
    }

    read_task.await;

    Ok(())
}
