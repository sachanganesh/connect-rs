# connect-rs

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/connect.svg
[crates-url]: https://crates.io/crates/connect
[docs-badge]: https://docs.rs/connect/badge.svg
[docs-url]: https://docs.rs/connect
 
This Rust crate provides a simple, brokerless message-queue abstraction over asynchronous
network streams. It guarantees ordered message delivery and reception, and both TCP and TLS
transports are supported.

## Examples

````rust
// create a client connection to the server
let mut conn = Connection::tcp_client(ip_address).await?;

// construct a new message
let msg = String::from("Hello world!");
let envelope: ConnectDatagram = ConnectDatagram::with_tag(65535, msg.into_bytes())?;

// send a message to the server
conn.writer().send(envelope).await?;

// wait for the echo-server to reply with an echo
if let Some(mut envelope) = conn.reader().next().await {
    // take the message payload from the envelope
    let data: Vec<u8> = envelope.data().to_vec();

    // reconstruct the original message
    let msg = String::from_utf8(data)?;
    assert_eq!("Hello world!", msg.as_str());
}
````

In addition to the [crate documentation](https://docs.rs/connect/latest/connect/), please use
the provided [example programs](https://github.com/sachanganesh/connect-rs/tree/main/examples)
as a practical reference for crate usage.

- TCP
    - [TCP Echo Server](https://github.com/sachanganesh/connect-rs/tree/main/examples/tcp-echo-server)
    - [TCP Client](https://github.com/sachanganesh/connect-rs/tree/main/examples/tcp-client)
- TLS (enable `tls` feature flag)
    - [TLS Echo Server](https://github.com/sachanganesh/connect-rs/tree/main/examples/tls-echo-server)
    - [TLS Client](https://github.com/sachanganesh/connect-rs/tree/main/examples/tls-client)

## Why?

When building networked applications, developers shouldn't have to focus on repeatedly solving
the problem of reliable, ordered message delivery over byte-streams. By using a message
queue abstraction, crate users can focus on core application logic and leave the low-level
networking and message-queue guarantees to the abstraction.

Connect provides a `ConnectionWriter` and `ConnectionReader` interface to concurrently send
and receive messages over a network connection. Each user-provided message is prefixed by 8
bytes, containing a size-prefix (4 bytes), version field (2 bytes), and tag field (2 bytes).
The size-prefix and version field are used internally to deserialize messages received from the
network connection. The tag field is intended for crate users to label the message for a
recipient, although the library leaves that up to the user's discretion.

Library users must serialize their custom messages into bytes (`Vec<u8>`), prior to
constructing a `ConnectDatagram`, which can then be passed to a `ConnectionWriter`.
Consequently, `ConnectionReader`s will return `ConnectDatagram`s containing the message payload
(`Vec<u8>` again) to the user to deserialize.

Requiring crate users to serialize data before constructing a datagram may appear redundant, but
gives the developer the freedom to use a serialization format of their choosing. This means that
library users can do interesting things such as:

- Use the tag field to signify which serialization format was used for that message
- Use the tag field to signify the type of message being sent

## Feature Flags

- `tls`: enables usage of tls transport functionality

## Feature Status

| Feature                                             	| Status 	|
|-----------------------------------------------------	|--------	|
| [TCP Client](examples/tcp-client)      	            |    ✓   	|
| [TCP Server](examples/tcp-echo-server) 	            |    ✓   	|
| [TLS Client](examples/tls-client)      	            |    ✓   	|
| [TLS Server](examples/tls-echo-server) 	            |    ✓   	|
| SCTP Client                                         	|        	|
| SCTP Server                                         	|        	|
| DTLS-SCTP Client                                    	|        	|
| DTLS-SCTP Server                                    	|        	|

## Contributing

This crate gladly accepts contributions. Please don't hesitate to open issues or PRs.
