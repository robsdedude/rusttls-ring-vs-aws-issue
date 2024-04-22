# Minimal reproducer for rustls issue.

Parts of the code taken and adjusted from https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo

## Setup
> **IMPORTANT**  
> Add `127.0.0.1     thehost` to you `/etc/hosts` file.

Linux Mint 21
go version go1.18.1 linux/amd64
cargo 1.77.2 (e52e36006 2024-03-26)
rustc 1.77.2 (25ef9e3d8 2024-04-09)

## Run
In terminal 1
```bash
go run server.go
```

In terminal 2
```bash
# works fine
RING=y cargo run --bin client

# fails
RING=n cargo run --bin client
```

## Observations
 * I couldn't reproduce the issue with a rustls powered server.
   Regardless of the `CryptoProvider` used.
 * The TLS version on the serverside had to be fixed to `1.2` for the failure to occur.

### With `ring=y`
Output server:
```bash
$ go run server.go
TLS, listening on 0.0.0.0:6666 with cert certs/server_thehost.pem
TLS, client connected from 127.0.0.1:40086, waiting for Bolt handshake
Client connected with correct Bolt handshake
```

Output client (stripped log entries):
```bash
$ RING=y cargo run --bin client
   Compiling rustls-test v0.1.0 (/home/rouven/seafile/code/rust-test)
    Finished dev [unoptimized + debuginfo] target(s) in 1.75s
     Running `target/debug/client`
Successfully connected to server in port 6666
Sent Hello, awaiting reply...
Reply is ok!
Terminated. 
```

### With `ring=n`
Output server:
```bash
$ go run server.go
TLS, listening on 0.0.0.0:6666 with cert certs/server_thehost.pem
TLS, client connected from 127.0.0.1:35314, waiting for Bolt handshake
Failed to receive Bolt handshake
remote error: tls: error decrypting message
exit status 255
```

Output client (stripped log entries):
```bash
$ RING=n cargo run --bin client
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/client`
Successfully connected to server in port 6666
thread 'main' panicked at src/bin/client.rs:59:35:
called `Result::unwrap()` on an `Err` value: Custom { kind: InvalidData, error: InvalidCertificate(BadSignature) }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
