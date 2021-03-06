
extern crate redis;
use redis::{Commands, RedisResult};
use crate::routes::Simulation;

fn get_connection() -> redis::RedisResult<redis::Connection> {
    let client = redis::Client::open("redis://redis-master/")?;
    client.get_connection()
}

#[doc = "Function for requesting a new Simulation id from the Redis DB"]
pub fn get_new_simulation_id() -> RedisResult<u64> {
    let mut conn = get_connection()?;
    conn.incr("models", 1)
}

#[doc = "Function for writing a Simulation into a Redis DB"]
pub fn write_simulation(key: &String, value: &Simulation) -> Result<(), redis::RedisError> {
    let mut conn = get_connection()?;
    match serde_json::to_string(value) {
        Ok(value_str) => conn.set(key, value_str),
        Err(e) => Err((redis::ErrorKind::IoError, "".into(), e.to_string()).into())
    }
}

use redis::RedisError;

#[doc = "Function for reading a Simulation from a Redis DB"]
pub fn read_simulation(key: u64) -> Result<Simulation, RedisError> {
    let mut conn = get_connection()?;
    match conn.get::<u64, Vec<u8>>(key) {
        Ok(value_utf8) => {
            if value_utf8.len() == 0 {
                return Err(RedisError::from((redis::ErrorKind::IoError,
                    "Simulation does not exist in database".into(), key.to_string())))
            }
            match String::from_utf8(value_utf8) {
                Ok(value_string) => match serde_json::from_str(&value_string) {
                    Ok(sim) => Ok(sim),
                    Err(e) => {
                        let error_string = format!("value: {} error: {}", value_string, e.to_string());
                        Err(RedisError::from((redis::ErrorKind::IoError,
                            "Could not convert string to Simulation! ", error_string.into())))
                    }
                }
                Err(e) => return Err((redis::ErrorKind::IoError,
                    "Could not convert utf8 from Redis into string: ".into(), e.to_string()).into())
            }
        }
        Err(e) => return Err((redis::ErrorKind::IoError,
            "Could not fetch item from Redis: ".into(), e.to_string()).into())
    }
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

#[doc = "Utility function for writing an int value to a key in a Redis DB"]
pub fn write_string(key: &String, value: String) -> redis::RedisResult<()> {
    let mut conn = get_connection()?;
    conn.set(key, value)
}

#[doc = "Utility function for reading an int value from a key in a Redis DB"]
pub fn read_string(key: &String) -> redis::RedisResult<String> {
    let mut conn = get_connection()?;
    conn.get(key)
}

