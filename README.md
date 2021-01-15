# connect

This crate provides a distributed message queue abstraction over asynchronous network streams.

By using a message queue, crate users can focus on sending and receiving messages between clients instead of low-level networking and failure recovery.

## Future Goals

- Documentation
- Connection pool (+ accounting for ordering of messages)
- Configurable policies for handling of non-registered (unexpected) message types
- Testing
- Benchmarking

## Feature Status

| Feature                                             	| Status 	|
|-----------------------------------------------------	|--------	|
| [TCP Client](examples/tcp-client)      	            |    ✓   	|
| [TCP Server](examples/tcp-echo-server) 	            |    ✓   	|
| UDP Client                                          	|        	|
| UDP Server                                          	|        	|
| [TLS Client](examples/tls-client)      	            |    ✓   	|
| [TLS Server](examples/tls-echo-server) 	            |    ✓   	|
| QUIC Client                                         	|        	|
| QUIC Server                                         	|        	|
| SCTP Client                                         	|        	|
| SCTP Server                                         	|        	|
| DTLS-SCTP Client                                    	|        	|
| DTLS-SCTP Server                                    	|        	|
| Kafka Client                                        	|        	|
| RMQ Client                                          	|        	|
| SQS Client                                          	|        	|
| NSQ Client                                          	|        	|
