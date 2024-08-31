use crate::resp::RespValue;
use std::collections::HashMap;

// SET - Insert a new key-value pair into the hashmap
pub fn set(storage: &mut HashMap<String, String>, key: String, value: String) -> RespValue {
    storage.insert(key, value);

    for (key, value) in storage {
        println!("{}: {}", key, value)
    }

    RespValue::Text("OK".to_string())
}

// GET - Get value from hashmap (if exists) or return null given a key
pub fn get(storage: &HashMap<String, String>, key: String) -> RespValue {
    match storage.get(&key) {
        Some(v) => {
            println!("({}, {})", key, v);
            RespValue::BulkString(v.clone())
        }
        None => RespValue::Null,
    }
}

// PING - Return "PONG"
pub fn ping() -> RespValue {
    RespValue::Text("PONG".to_string())
}
