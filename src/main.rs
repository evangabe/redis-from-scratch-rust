mod resp;

use anyhow::{Ok, Result};
use resp::RespValue;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    // Open TCP connection at 6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // Await packets
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}

struct RespConnection {
    stream: TcpStream,
    buffer: [u8; 512],
}

impl RespConnection {
    pub fn new(stream: TcpStream) -> RespConnection {
        RespConnection {
            stream,
            buffer: [0; 512],
        }
    }

    pub async fn read(&mut self) -> Result<Option<RespValue>> {
        let bytes_read = self.stream.read(&mut self.buffer).await.unwrap();
        if bytes_read == 0 {
            return Ok(None);
        }

        let (value, _) = resp::parse(&self.buffer)?;
        Ok(Some(value))
    }

    pub async fn write(&mut self, mut value: RespValue) -> Result<()> {
        self.stream.write(value.serializer().as_bytes()).await?;
        Ok(())
    }
}

async fn handle_connection(stream: TcpStream) {
    let mut connection = RespConnection::new(stream);
    loop {
        let value = connection.read().await.unwrap();
        println!("Got value {:?}", value);

        let response = if let Some(v) = value {
            let (cmd, args) = extract_command(v).unwrap();
            match cmd.as_str() {
                "ping" => RespValue::Text("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };
        println!("Sending value {:?}", response);
        connection.write(response).await.unwrap();
    }
}

fn extract_command(value: RespValue) -> Result<(String, Vec<RespValue>)> {
    match value {
        RespValue::Array(arr) => Ok((
            unpack_bulk_str(arr.first().unwrap().clone())?,
            arr.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Expected command array".to_string())),
    }
}

fn unpack_bulk_str(value: RespValue) -> Result<String> {
    match value {
        RespValue::BulkString(bs) => Ok(bs.to_lowercase()),
        _ => Err(anyhow::anyhow!("Expected bulk string".to_string())),
    }
}
