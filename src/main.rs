#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

// fn handle_client(mut stream: TcpStream) {
//     let mut buf = [0; 512];
//     stream.read(&mut buf).unwrap();
//     stream.write(b"+PONG\r\n").unwrap();
// }

fn main() {
    println!("Opening TCP server . . .");
    // Open TCP connection at 6379
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Await packets
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection!");
                // handle_client(stream);
                let mut buf = [0; 512];
                stream.read(&mut buf).unwrap();
                stream.write(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
