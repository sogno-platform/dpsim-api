//!
//! # DPSIM Service Rest API for controlling DPSim analyzer
//! Author : Richard Marston <rmarston@eonerc.rwth-aachen.de>
//!
//! ## Table of endpoints
//!
//! | Endpoint                    | Method | Description        | Implementation              | Parameters                      | Returns                |
//! |-----------------------------|--------|--------------------|-----------------------------|---------------------------------|------------------------|
//! | /simulation                 | POST   | Add a simulation   | [`post_simulation`][post_s] | [`SimulationForm`][s_f_s]       | [`Simulation`][sim]    |
//! | /simulation                 | GET    | List simulations   | [`get_simulations`][get_s]  | None                            | [ [`Simulation`][sim] ]|
//! | /simulation/ \[id]          | GET    | Simulation details | [`todo!`]                   | None                            | [`Simulation`][sim]    |
//! | /simulation/ \[id] /results | GET    | Simulation results | [`todo!`]                   | None                            | plain text             |
//! | /simulation/ \[id] /logs    | GET    | Simulation logs    | [`todo!`]                   | None                            | plain text             |
//! | /debug                      | GET    | DPsim-api debug    | [`todo!`]                   | None                            | plain text             |
//!
//! [post_s]: routes::post_simulation()
//! [get_s]: routes::get_simulations()
//! [s_f_s]: routes::SimulationForm
//! [sim]: routes::Simulation

// stop rustdoc complaining that I'm linking
// to the routes module, which is privately
// owned by this one (main.rs)
#![allow(rustdoc::private_intra_doc_links)]

#[macro_use]
extern crate rocket;

mod routes;
mod file_service;
mod amqp;
#[cfg(not(test))] mod db;
use rocket_dyn_templates::Template;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}

#[rocket::main]
#[doc = "The main entry point for Rocket" ]
async fn main() -> Result <(), rocket::Error> {

    rocket::build()
        .register("/", catchers![routes::incomplete_form])
        .mount("/", routes::get_routes())
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .attach(Template::fairing())
        .launch()
        .await
}

#[cfg(test)]
mod db {
    use redis::RedisResult;
    use crate::routes::{Simulation, SimulationType};
    pub fn get_new_simulation_id() -> RedisResult<u64> {
        Ok(1)
    }
    pub fn write_simulation(_key: &String, _value: &Simulation) -> redis::RedisResult<()> {
        Ok(())
    }
    pub fn read_simulation(_key: u64) -> redis::RedisResult<Simulation> {
        Ok(Simulation {
               error:           "".to_owned(),
               load_profile_id: "".into(),
               model_id:        "1".to_string(),
               results_id:      "1".to_string(),
               results_data:    "1".to_string(),
               simulation_id:   1,
               simulation_type: SimulationType::Powerflow
        })
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
