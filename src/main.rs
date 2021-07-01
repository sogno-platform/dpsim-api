//!
//! # DPSIM Service Rest API for controlling DPSim analyzer
//! Author : Richard Marston <rmarston@eonerc.rwth-aachen.de>
//! ## Table of endpoints
//!
//! | Endpoint                    | Method | Description        | Implementation        | Parameters         | Returns           |
//! |-----------------------------|--------|--------------------|-----------------------|--------------------|-------------------|
//! | /simulation                 | POST   | Add a simulation   | [`routes::post_simulation()`] | [`SimulationForm`] | [`Simulation`]    |
//! | /simulation                 | GET    | List simulations   | [`routes::get_simulation()`]  | None               | [ [`Simulation`] ]|
//! | /simulation/ \[id]          | GET    | Simulation details | [`todo!`]             | None               | [`Simulation`]    |
//! | /simulation/ \[id] /results | GET    | Simulation results | [`todo!`]             | None               | plain text        |
//! | /simulation/ \[id] /logs    | GET    | Simulation logs    | [`todo!`]             | None               | plain text        |
//! | /debug                      | GET    | DPsim-api debug    | [`todo!`]             | None               | plain text        |
//!


#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate doc_comment;

use std::path::PathBuf;
use rocket::fs::TempFile;
use rocket::form::Form;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::fs::File;
use std::ffi::OsStr;
use std::path::Path;
use std::io::prelude::*;
mod routes;
#[cfg(not(test))] mod amqp;
#[cfg(not(test))] mod db;

#[derive(Serialize, Deserialize, FromForm)]
pub struct Simulation {
    error: String,
    load_profile_data: Vec <String>,
    model_id:        u64,
    simulation_id:   u64,
    simulation_type: String,
}

pub type SimulationJson = Json<Simulation>;

/// # Form for submitting a new Simulation
///
/// ## Parameters:
/// * simulation_type
///   - String
///   - must be one of "Powerflow", "Outage"
/// * load_profile_data
///   - ZIP file
///   - load profile data in CSV format
/// * model_id
///   - Integer
///   - must be a valid id that exists in the associated CIM service
#[derive(FromForm, Debug)]
pub struct SimulationForm<'f> {
    simulation_type: String,
    load_profile_data: TempFile<'f>,
    model_id: u64
}

enum SimulationType {
    Powerflow,
    Outage
}

fn osstr_to_string(str: &OsStr) -> String {
    let the_str = str.to_str().unwrap();
    String::from(the_str)
}

fn read_zip(reader: impl Read + Seek) -> zip::result::ZipResult<Vec<String>> {
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut files = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        if file.is_file() {
            let path = Path::new(file.name());
            let stringy = match path.file_name() {
                Some(file_name) => osstr_to_string(file_name),
                None => "".into()
            };
            files.push(stringy);
        }
    }

    Ok(files)
}

async fn parse_simulation_form(mut form: Form<SimulationForm<'_>>) -> Result<Json<Simulation>, String>{
    let tmp_dir = PathBuf::from("/tmp/");
    let simulation_id = 0;
    let save_name = tmp_dir.join(format!("simulation_{}", simulation_id));
    match form.load_profile_data.persist_to(&save_name).await {
        Ok (()) => {
            let f = File::open(save_name).unwrap();
            match read_zip(f) {
                Ok(data) => {
                    let simulation = Simulation {
                        error: "".to_string(),
                        simulation_id: simulation_id,
                        simulation_type: form.simulation_type.clone(),
                        model_id: form.model_id,
                        load_profile_data: data
                    };
                    match db::write_simulation(&simulation_id.to_string(), &simulation) {
                        Ok(()) => Ok(Json(simulation)),
                        Err(e) => Err(format!("Could not write to db: {}", e.to_string()))
                    }
                }
                Err(e) => Err(format!("Failed to read zip: {}", e.to_string()))
            }
        },
        Err(e) => Err(format!("Failed to store load profile data: {}", e))
    }
}


#[rocket::main]
#[doc = "The main entry point for Rocket" ]
async fn main() -> Result <(), rocket::Error> {

    rocket::build()
        .register("/", catchers![routes::incomplete_form])
        .mount("/", routes::get_routes())
        .launch()
        .await
}

#[cfg(test)]
mod amqp {
    use lapin::Result;
    pub async fn publish(simulation: &super::Simulation) -> Result<()> {
        Ok(())
    }
}
#[cfg(test)]
mod db {
    pub fn write_simulation(_key: &String, _value: &super::Simulation) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn write_u64(_key: &String, _value: u64) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn read_u64(_key: &String) -> redis::RedisResult<u64> {
        Ok(36)
    }
}
#[cfg(test)]
mod tests;
