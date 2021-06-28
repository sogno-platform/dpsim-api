
extern crate redis;
use redis::Commands;

fn get_connection() -> redis::RedisResult<redis::Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_connection()
}

#[doc = "Utility function for writing an int value to a key in a Redis DB"]
pub fn write_u64(key: &String, value: u64) -> redis::RedisResult<()> {
    let mut conn = get_connection()?;
    conn.set(key, value)
}

#[doc = "Utility function for reading an int value from a key in a Redis DB"]
pub fn read_u64(key: &String) -> redis::RedisResult<u64> {
    let mut conn = get_connection()?;
    conn.get(key)
}

