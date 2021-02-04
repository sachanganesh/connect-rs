# connect-rs

This Rust crate provides a reliable, fault-tolerant, and brokerless message-queue abstraction over
asynchronous network streams.

## Why?
When building networked applications, developers shouldn't have to focus on repeatedly solving
the problem of reliable, fault-tolerant message delivery over byte-streams. By using a message
queue abstraction, crate users can focus on core application logic and leave the low-level
networking and message-queue guarantees to the abstraction.

# Protobuf
This crate relies on the use of [Protocol Buffers](https://developers.google.com/protocol-buffers)
due to it being widely adopted and industry-proven. All messages are Protobuf messages that
are packed into a Protobuf `Any` type and then sent over the wire. Message recipients must
decide what Protobuf message type it is, and correspondingly unpack the `Any` into a particular
message type.

# Examples
Please use the [examples](https://github.com/sachanganesh/connect-rs/tree/main/examples)
provided to help understand crate usage.


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
