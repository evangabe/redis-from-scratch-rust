use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    println!("Opening TCP server . . .");
    // Open TCP connection at 6379
    let tcp_addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(tcp_addr).await.unwrap();

    // Await packets
    loop {
        let stream = listener.accept().await;

        match stream {
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
                        // This is where the high-performance Redis parser will sit

                        // Must separate commands
                        // Must process command arguments if they exists
                        // including strings and arrays that vary in length
                        // Throw errors for false commands or invalid arguments
                        // for line in request.lines() {
                        match parse_resp(request) {
                            Ok((resp, _rest)) => {
                                // Handle RESP parts here
                                println!("[RESP OUTPUT]:\n {:?}", resp);

                                stream.write(b"+PONG\r\n").await.unwrap();
                            }
                            Err(e) => {
                                println!("Error parsing RESP: {}", e);
                            }
                        }
                        // }
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

#[derive(Debug)]
enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespType>),
}

fn parse_resp(input: &str) -> Result<(RespType, &str), String> {
    println!("[RESP Input]:\n{:?}", input);

    // Split the first line and the remainder
    if let Some((content, mut rest)) = input.split_once("\r\n") {
        println!("[RESP Line]:\n{}", content);

        // Match prefix
        match content.chars().next() {
            // Parse array
            Some('*') => {
                // Parse array length
                let length = content[1..] // Skip the '*' prefix
                    .parse::<usize>()
                    .map_err(|e| format!("Failed to parse array length: {}", e))?;

                let mut elements = Vec::with_capacity(length);

                // Recursively parse each element in the array
                for _ in 0..length {
                    let (element, new_rest) = parse_resp(rest)?;
                    elements.push(element);
                    rest = new_rest;
                }

                Ok((RespType::Array(elements), rest))
            }
            // Parse bulk string
            Some('$') => {
                let length = content[1..]
                    .parse::<i64>()
                    .map_err(|e| format!("Failed to parse bulk string length: {}", e))?;

                if length == -1 {
                    Ok((RespType::BulkString(None), rest))
                } else {
                    let (data, new_rest) = rest.split_at(length as usize);
                    if new_rest.starts_with("\r\n") {
                        Ok((RespType::BulkString(Some(data.to_string())), &new_rest[2..]))
                    } else {
                        Err("Malformed Bulk String".to_string())
                    }
                }
            }
            // Parse simple string
            Some('+') => {
                let text = content[1..].to_string();
                Ok((RespType::SimpleString(text), rest))
            }
            _ => Err("Invalid RESP identifier".to_string()),
        }
    } else {
        Ok((RespType::Error("input not splitable".to_string()), ""))
    }
}
