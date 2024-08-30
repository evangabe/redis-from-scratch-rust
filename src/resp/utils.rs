use anyhow::{Ok, Result};

pub fn split_at_crlf(buffer: &[u8]) -> Result<(i64, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let length = parse_int(line)?;
        return Ok((length, len + 1));
    }
    Err(anyhow::anyhow!("Invalid bulk string {:?}", buffer))
}

pub fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    None
}

pub fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}
