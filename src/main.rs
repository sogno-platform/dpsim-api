//!
//! # DPSIM Service Rest API for controlling DPSim analyzer
//! Author : Richard Marston <rmarston@eonerc.rwth-aachen.de>
//!
//! ## Table of endpoints
//!
//! | Endpoint                    | Method | Description        | Implementation              | Parameters                      | Returns           |
//! |-----------------------------|--------|--------------------|-----------------------------|---------------------------------|-------------------|
//! | /simulation                 | POST   | Add a simulation   | [`post_simulation`][post_s] | [`SimulationFormStruct`][s_f_s] | [`Simulation`]    |
//! | /simulation                 | GET    | List simulations   | [`get_simulations`][get_s]  | None                            | [ [`Simulation`] ]|
//! | /simulation/ \[id]          | GET    | Simulation details | [`todo!`]                   | None                            | [`Simulation`]    |
//! | /simulation/ \[id] /results | GET    | Simulation results | [`todo!`]                   | None                            | plain text        |
//! | /simulation/ \[id] /logs    | GET    | Simulation logs    | [`todo!`]                   | None                            | plain text        |
//! | /debug                      | GET    | DPsim-api debug    | [`todo!`]                   | None                            | plain text        |
//!
//! [post_s]: routes::post_simulation()
//! [get_s]: routes::get_simulations()
//! [s_f_s]: routes::SimulationFormStruct

// stop rustdoc complaining that I'm linking
// to the routes module, which is privately
// owned by this one (main.rs)
#![allow(rustdoc::private_intra_doc_links)]

#[macro_use]
extern crate rocket;

use rocket::serde::{Deserialize, Serialize};
mod routes;
mod file_service;
mod amqp;
#[cfg(not(test))] mod db;
use rocket_dyn_templates::Template;
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[doc = "Struct for encapsulation Simulation details"]
pub struct Simulation {
    error: String,
    load_profile_data: Vec <String>,
    model_id:          String,
    simulation_id:     u64,
    simulation_type:   SimulationType,
}

#[doc = "Enum for the various Simulation types"]
#[derive(FromFormField, Serialize, Deserialize, Debug, Copy, Clone, JsonSchema)]
enum SimulationType {
    Powerflow,
    Outage
}

impl Default for SimulationType {
    fn default() -> Self {
        SimulationType::Powerflow
    }
}

#[doc = "String conversion for the various Simulation types"]
impl SimulationType {
    fn to_string(&self) -> String {
        match &*self {
            SimulationType::Powerflow => "Powerflow".to_owned(),
            SimulationType::Outage    => "Outage".to_owned()
        }
    }
}

#[rocket::main]
#[doc = "The main entry point for Rocket" ]
async fn main() -> Result <(), rocket::Error> {

    rocket::build()
        .register("/", catchers![routes::incomplete_form])
        .mount("/", routes::get_routes())
        .attach(Template::fairing())
        .launch()
        .await
}

#[cfg(test)]
mod db {
    use redis::RedisResult;
    use crate::{Simulation, SimulationType};
    pub fn get_new_simulation_id() -> RedisResult<u64> {
        Ok(1)
    }
    pub fn write_simulation(_key: &String, _value: &super::Simulation) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn read_simulation(_key: u64) -> redis::RedisResult<Simulation> {
        Ok(Simulation { error: "".to_owned(), load_profile_data: [].to_vec(), model_id: "1".to_string(), simulation_id: 1, simulation_type: SimulationType::Powerflow })
    }
    pub fn write_u64(_key: &String, _value: u64) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn read_u64(_key: &String) -> redis::RedisResult<u64> {
        Ok(36)
    }
    pub fn write_string(_key: &String, _value: String) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn read_string(_key: &String) -> redis::RedisResult<String> {
        Ok("1".to_string())
    }
}
#[cfg(test)]
mod tests;
