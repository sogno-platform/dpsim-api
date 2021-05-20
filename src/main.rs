//! DPSIM Service Rest API
//!
//!
//! [`LINK`]: http://example.com

//! # Hello World
//!
//! ```bash
//! $ curl www.example.com/ui/analysis/id 
//! }
//! ```
//!
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate redis;

use rocket::data::TempFile;
use rocket::form::Form;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use redis::Commands;
use log::{info, warn};

fn write_u32(key: &String, value: u32) -> Result<redis::RedisResult<()>, String> {
    let client = match redis::Client::open("redis://127.0.0.1/"){
        Ok(_client) => _client,
        Err(_error) => return Err("Can't open redis client.".into())
    };
    match client.get_connection() {
        Ok(mut con) => match con.set(key, value) {
            Ok(ok) => return Ok(redis::RedisResult::Ok(ok)),
            Err(_e) => return Err(_e.to_string())
        }
        Err(_error) => return Err("Can't connect to redis db.".into())
    }
}

fn read_u32(key: &String) -> redis::RedisResult<u32> {
    match redis::Client::open("redis://127.0.0.1/") {
        Ok(client) => {
            match client.get_connection() {
                Ok(mut con) => con.get(key),
                Err(e) => return Err(e)
            }
        }
        Err(e) => return Err(e)
    }
}

/// Example
/// ```rust
/// main() -> Result<(), std::num::ParseIntError> {
/// let fortytwo = "42".parse::<u32>()?;
/// println!("{} + 10 = {}", fortytwo, fortytwo+10);
///     Ok(())
/// }
/// 

#[derive(Serialize, Deserialize, FromForm)]
struct Analysis {
    id: u32
}

#[derive(Serialize, Deserialize)]
struct Status {
    status: u32
}

type AnalysisJson = Result<Json<Analysis>, String>;

/// Get all network analyses
/// Parameters: none
/// Responses:
/// 200 - Content is array of Analysis json objects
/// Error code
#[get("/analysis")]
fn analysis() -> AnalysisJson {
    match write_u32(&String::from("happy"), 36u32) {
        Ok(_) => (),
        Err(e) => return Err(e)
    };
    match read_u32(&String::from("happy")) {
        Ok(i) => Ok(Json(Analysis { id: i })),
        Err(_) => Err("OH NO".into())
    }
}

/// Add a new analysis
/// Parameters:
/// multipart/form-data:
///     name     - String
///     type     - String
///     modelid  - int
///     csv_data - zip file of load profile data
#[derive(FromForm)]
pub struct AddAnalysisForm<'f> {
    id: u32,
    file: TempFile<'f>,
}

#[post("/analysis", format = "multipart/form-data", data = "<form>")]
pub async fn post_analysis(mut form: Form<AddAnalysisForm<'_>>) -> std::io::Result<()> {
    form.file.persist_to("/tmp/complete/file.jpg").await?;
    Ok(())
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![analysis, post_analysis])
        .launch()
        .await;
}

