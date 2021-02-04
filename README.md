# connect-rs

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/connect.svg
[crates-url]: https://crates.io/crates/connect
[docs-badge]: https://docs.rs/connect/badge.svg
[docs-url]: https://docs.rs/connect

This Rust crate provides a simple brokerless message-queue abstraction over asynchronous network
streams.

## Why?
When building networked applications, developers shouldn't have to focus on repeatedly solving
the problem of reliable, fault-tolerant message delivery over byte-streams. By using a message
queue abstraction, crate users can focus on core application logic and leave the low-level
networking and message-queue guarantees to the abstraction.

## Examples
Please use the [examples](https://github.com/sachanganesh/connect-rs/tree/main/examples)
provided to help understand crate usage.

## Protobuf
This crate relies on the use of [Protocol Buffers](https://developers.google.com/protocol-buffers)
due to it being widely adopted and industry-proven. All messages are Protobuf messages that
are packed into a Protobuf `Any` type and then sent over the wire. Message recipients must
decide what Protobuf message type it is, and correspondingly unpack the `Any` into a particular
message type.

Protobuf was chosen when the library hit a roadblock with Rust's `TypeId` potentially not being
consistent between Rust compiler versions. The crate requires a consistent way to determine what
type of message is received, so it can appropriately deserialize the message from network bytes.
Until the Rust ecosystem around reflection improves, the crate will use Protobuf to fill the
void.

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
