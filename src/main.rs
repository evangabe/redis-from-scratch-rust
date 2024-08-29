#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    println!("Opening TCP server . . .");
    // Open TCP connection at 6379
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Await packets
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 512];
                loop {
                    let read_count = stream.read(&mut buf).unwrap();
                    if read_count == 0 {
                        break;
                    }

                    let request = std::str::from_utf8(&buf[..read_count]).unwrap();
                    for line in request.lines() {
                        if line == "PING" {
                            stream.write(b"+PONG\r\n").unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
