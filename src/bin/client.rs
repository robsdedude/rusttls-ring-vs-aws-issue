use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str::from_utf8;
use std::sync::Arc;

use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let root_store = {
        let path = "certs/serverRoot.crt";
        let mut root_store = RootCertStore::empty();
        let file = File::open(path)
            .map_err(|e| format!("failed to open certificate(s) path {path:?}: {e}"))
            .unwrap();
        let mut reader = BufReader::new(file);
        for cert_res in rustls_pemfile::certs(&mut reader) {
            let cert = cert_res
                .map_err(|e| format!("failed to load certificate(s) from {path:?}: {e}"))
                .unwrap();
            root_store
                .add(cert)
                .map_err(|e| {
                    format!("failed to add certificate(s) from {path:?} to root store: {e}")
                })
                .unwrap();
        }
        root_store
    };
    if &env::var("RING").unwrap_or_default() == "y" {
        rustls::crypto::ring::default_provider()
            .install_default()
            .unwrap();
    } else {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .unwrap();
    }
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    match TcpStream::connect("thehost:6666") {
        Ok(stream) => {
            println!("Successfully connected to server in port 6666");
            let client_connection =
                ClientConnection::new(Arc::new(client_config), "thehost".try_into().unwrap())
                    .unwrap();
            let mut stream = StreamOwned::new(client_connection, stream);

            let msg =
                b"\x60\x60\xB0\x17\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            stream.write_all(msg).unwrap();
            println!("Sent Hello, awaiting reply...");

            let mut data = [0u8; 4];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == b"\x00\x00\x00\x00" {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(dbg!(&data)).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
            stream.flush().unwrap();
            stream.conn.send_close_notify();
            stream.flush().unwrap();
            let _ = stream.sock.shutdown(Shutdown::Both);
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
