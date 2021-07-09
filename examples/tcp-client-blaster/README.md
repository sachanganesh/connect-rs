# connect tcp-client-blaster example

This example program will:

1. Establish a connection with a TCP server
2. Send a number of `String` messages to the server
3. Wait for `String` messages reply from the server
4. Validate that the messages are received in the correct order

## Usage

```
export RUST_LOG=info
cargo run <ip-address-to-connect-to>
```

## Example Usage

```
export RUST_LOG=info
cargo run localhost:5678
```