use anyhow::{Ok, Result};
mod utils;

#[derive(Clone, Debug)]
pub enum RespValue {
    Text(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespValue>),
}

impl RespValue {
    pub fn serializer(&mut self) -> String {
        match self {
            RespValue::Text(s) => format!("+{}\r\n", s),
            RespValue::Integer(i) => format!("-{}\r\n", i),
            RespValue::BulkString(bs) => format!("${}\r\n{}\r\n", bs.chars().count(), bs),
            _ => panic!("Unsupported type for serialization!"),
        }
    }
}

pub fn parse(buffer: &[u8]) -> Result<(RespValue, usize)> {
    match buffer[0] as char {
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        '+' => parse_text(buffer),
        '-' => parse_integer(buffer),
        _ => Err(anyhow::anyhow!("Invalid RESP identifier".to_string())),
    }
}

fn parse_text(buffer: &[u8]) -> Result<(RespValue, usize)> {
    if let Some((line, len)) = utils::read_until_crlf(buffer) {
        return Ok((RespValue::Text(String::from_utf8(line.to_vec())?), len));
    }
    Err(anyhow::anyhow!("Malformed text string".to_string()))
}

fn parse_integer(buffer: &[u8]) -> Result<(RespValue, usize)> {
    if let Some((line, len)) = utils::read_until_crlf(buffer) {
        return Ok((RespValue::Integer(utils::parse_int(line)?), len));
    }
    Err(anyhow::anyhow!("Malformed integer".to_string()))
}

fn parse_array(buffer: &[u8]) -> Result<(RespValue, usize)> {
    let (arr_len, mut bytes_consumed) = utils::split_at_crlf(&buffer)?;

    let mut items = vec![];
    for _ in 0..arr_len {
        let (element, len) = parse(&buffer[bytes_consumed..])?;
        items.push(element);
        bytes_consumed += len;
    }

    Ok((RespValue::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: &[u8]) -> Result<(RespValue, usize)> {
    let (str_len, bytes_consumed) = utils::split_at_crlf(&buffer)?;
    let end_of_str = bytes_consumed + str_len as usize;

    Ok((
        RespValue::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_of_str].to_vec(),
        )?),
        end_of_str + 2,
    ))
}
