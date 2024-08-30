#[derive(Debug)]
pub enum RespType {
    Text(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespType>),
}

pub fn parse_resp(input: &str) -> Result<(RespType, &str), String> {
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
                Ok((RespType::Text(text), rest))
            }
            Some('-') => {
                let num = content[1..]
                    .parse::<i64>()
                    .map_err(|e| format!("Failed to parse bulk string length: {}", e))?;

                Ok((RespType::Integer(num), rest))
            }
            _ => Err("Invalid RESP identifier".to_string()),
        }
    } else {
        Ok((RespType::Error("input not splitable".to_string()), ""))
    }
}
