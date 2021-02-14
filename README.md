# connect-rs

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/connect.svg
[crates-url]: https://crates.io/crates/connect
[docs-badge]: https://docs.rs/connect/badge.svg
[docs-url]: https://docs.rs/connect

This Rust crate provides a simple, brokerless message-queue abstraction over asynchronous network
streams.

## Examples
Please use the [example programs](https://github.com/sachanganesh/connect-rs/tree/main/examples)
provided to help understand crate usage.

## Why?
When building networked applications, developers shouldn't have to focus on repeatedly solving
the problem of reliable, fault-tolerant message delivery over byte-streams. By using a message
queue abstraction, crate users can focus on core application logic and leave the low-level
networking and message-queue guarantees to the abstraction.

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
