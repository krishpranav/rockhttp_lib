extern crate base64;
extern crate notify;

use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::str;
use std::thread;

pub const RELOAD_PORT: u32 = 8129; 

fn parse_websocket_handshake(bytes: &[u8]) -> String {
    let request_string = str::from_utf8(&bytes).unwrap();
    let lines = request_string.split("\r\n");
    let mut sec_websocket_key = "";

    for line in lines {
        let parts: Vec<&str> = line.split(':').collect();
        if let "Sec-WebSocket-Key" = parts[0] {
            sec_websocket_key = parts[1].trim();
        }
    }

    let sec_websocket_accept = format!(
        "{}{}",
        sec_websocket_key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
    );
    let mut hasher = Sha1::new();
    hasher.input(sec_websocket_accept.as_bytes());
    let result = hasher.result();
    let bytes = base64::encode(&result);

    format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",bytes)
}

fn send_websocket_message<T: Write>(mut stream: T) -> Result<(), std::io::Error> {
    let payload_length = 0;

    stream.write_all(&[129])?; 
    let mut second_byte: u8 = 0;

    second_byte |= payload_length as u8;
    stream.write_all(&[second_byte])?;

    Ok(())
}

fn handle_websocket_handshake<T: Read + Write>(mut stream: T) {
    let header = crate::read_header(&mut stream);
    let response = parse_websocket_handshake(&header);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn watch_for_reloads(address: &str, path: &str) {
    use notify::{DebouncedEvent::*, RecommendedWatcher, RecursiveMode, Watcher};

    let websocket_address = format!("{}:{:?}", address, RELOAD_PORT);
    let listener = TcpListener::bind(websocket_address).unwrap();

    for stream in listener.incoming() {
        let path = path.to_owned();

        thread::spawn(move || {
            if let Ok(mut stream) = stream {
                handle_websocket_handshake(&mut stream);

                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher: RecommendedWatcher =
                    Watcher::new(tx, std::time::Duration::from_millis(10)).unwrap();
                watcher
                    .watch(Path::new(&path), RecursiveMode::Recursive)
                    .unwrap();

                loop {
                    match rx.recv() {
                        Ok(event) => {
                            let refresh = match event {
                                NoticeWrite(..) | NoticeRemove(..) | Remove(..) | Rename(..)
                                | Rescan => false,
                                Create(..) | Write(..) | Chmod(..) => true,
                                Error(..) => panic!(),
                            };

                            if refresh {
                                if send_websocket_message(&stream).is_err() {
                                    break;
                                };
                            }
                        }
                        Err(e) => println!("File watch error: {:?}", e),
                    };
                }
            }
        });
    }
}