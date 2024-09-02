use crate::{cmds, db::Db, resp};
use anyhow::{Ok, Result};
use atoi::atoi;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Duration,
};

pub struct RespConnection {
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

    pub async fn read(&mut self) -> Result<Option<resp::RespValue>> {
        let bytes_read = self.stream.read(&mut self.buffer).await.unwrap();
        if bytes_read == 0 {
            return Ok(None);
        }

        let (value, _) = resp::parse(&self.buffer)?;
        Ok(Some(value))
    }

    pub async fn write(&mut self, mut value: resp::RespValue) -> Result<()> {
        self.stream.write(value.serializer().as_bytes()).await?;
        Ok(())
    }
}

pub async fn handle_connection(stream: TcpStream, storage: &Db) {
    let mut connection = RespConnection::new(stream);

    loop {
        let value = connection.read().await.unwrap();

        let response = if let Some(v) = value {
            let (cmd, args) = extract_command(v).unwrap();
            match cmd.as_str() {
                "ping" => cmds::ping(),
                "echo" => args[0].clone(),
                "set" => cmds::set(
                    &storage,
                    unpack_bulk_str(args.get(0).unwrap().clone()).unwrap(),
                    unpack_bulk_str(args.get(1).unwrap().clone()).unwrap(),
                    args.get(3)
                        .map(|px| Duration::from_millis(unpack_u64(px.clone()).unwrap())),
                ),
                "get" => cmds::get(
                    &storage,
                    unpack_bulk_str(args.get(0).unwrap().clone()).unwrap(),
                ),
                "list" => cmds::list(
                    &storage,
                    args.get(0)
                        .map(|limit| unpack_u64(limit.clone()).unwrap() as usize),
                ),
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };
        connection.write(response).await.unwrap();
    }
}

fn extract_command(value: resp::RespValue) -> Result<(String, Vec<resp::RespValue>)> {
    match value {
        resp::RespValue::Array(arr) => Ok((
            unpack_bulk_str(arr.first().unwrap().clone())?,
            arr.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Expected command array".to_string())),
    }
}

fn unpack_bulk_str(value: resp::RespValue) -> Result<String> {
    match value {
        resp::RespValue::BulkString(bs) => Ok(bs.to_lowercase()),
        _ => Err(anyhow::anyhow!("Expected bulk string".to_string())),
    }
}

fn unpack_u64(value: resp::RespValue) -> Result<u64> {
    let str = unpack_bulk_str(value).unwrap();
    Ok(atoi::<u64>(&str.as_bytes()).unwrap())
}
