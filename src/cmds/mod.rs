use crate::db::Db;
use crate::resp::RespValue;
use tokio::time::Duration;

// SET - Insert a new key-value pair into the hashmap
pub fn set(storage: &Db, key: String, value: String, expiry: Option<Duration>) -> RespValue {
    storage.set(key, value, expiry);
    RespValue::Text("OK".to_string())
}

// GET - Get value from hashmap (if exists) or return null given a key
pub fn get(storage: &Db, key: String) -> RespValue {
    match storage.get(&key) {
        Some(v) => RespValue::BulkString(v.clone()),
        None => RespValue::Null,
    }
}

// PING - Return "PONG"
pub fn ping() -> RespValue {
    RespValue::Text("PONG".to_string())
}

pub fn list(storage: &Db, limit: Option<usize>) -> RespValue {
    let mut resp_entries: Vec<RespValue> = Vec::new();
    for val in storage.list(limit).iter() {
        resp_entries.push(RespValue::BulkString(val.to_string()));
    }
    RespValue::Array(resp_entries)
}
