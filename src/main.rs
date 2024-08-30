use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
mod resp;

#[tokio::main]
async fn main() {
    // Open TCP connection at 6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // Await packets
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                println!("accepted new connections\n");

                tokio::spawn(async move {
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }

                        let request = std::str::from_utf8(&buf[..read_count]).unwrap();
                        match resp::parse_resp(request) {
                            Ok((resp, _rest)) => {
                                // Handle RESP parts here
                                println!("[RESP OUTPUT]:\n {:?}", resp);
                                stream.write(b"+PONG\r\n").await.unwrap();
                            }
                            Err(e) => {
                                println!("Error parsing RESP: {}", e);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
