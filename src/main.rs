//!
//! # DPSIM Service Rest API for controlling DPSim analyzer
//! Author : Richard Marston <rmarston@eonerc.rwth-aachen.de>
//! ## Table of endpoints
//!
//! | Endpoint                    | Method | Description        | Implementation        | Parameters         | Returns           |
//! |-----------------------------|--------|--------------------|-----------------------|--------------------|-------------------|
//! | /simulation                 | POST   | Add a simulation   | [`post_simulation()`] | [`SimulationForm`] | [`Simulation`]    |
//! | /simulation                 | GET    | List simulations   | [`get_simulation()`]  | None               | [ [`Simulation`] ]|
//! | /simulation/ \[id]          | GET    | Simulation details | [`todo!`]             | None               | [`Simulation`]    |
//! | /simulation/ \[id] /results | GET    | Simulation results | [`todo!`]             | None               | plain text        |
//! | /simulation/ \[id] /logs    | GET    | Simulation logs    | [`todo!`]             | None               | plain text        |
//! | /debug                      | GET    | DPsim-api debug    | [`todo!`]             | None               | plain text        |
//!

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate redis;
#[macro_use]
extern crate doc_comment;

use std::path::PathBuf;
use rocket::data::TempFile;
use rocket::form::Form;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use redis::Commands;
use log::info;

#[doc = "Utility function for writing an int value to a key in a Redis DB"]
fn write_u64(key: &String, value: u64) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut conn = client.get_connection()?;
    conn.set(key, value)
}

#[doc = "Utility function for reading an int value from a key in a Redis DB"]
fn read_u64(key: &String) -> redis::RedisResult<u64> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut conn = client.get_connection()?;
    conn.get(key)
}

#[derive(Serialize, Deserialize, FromForm)]
pub struct Simulation {
    simulation_id:   u64,
    simulation_type: String,
    model_id:        u64
}

pub type SimulationJson = Json<Simulation>;

macro_rules! create_api_with_doc{
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[post($route_path:tt, format = $content_type:tt, data = "<form>")]
        $(#[$attr:meta])*
        pub async fn $name:ident( $(mut $arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        doc_comment! {
            concat!(
                $description, "\n",
                " * Content-Type: ", $content_type, "\n",
            ),
            #[ doc = "### Example request:" ]
            #[ doc = "```bash" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[post($route_path, format = $content_type, data = "<form>")]
            $( #[$attr] )*
            pub async fn $name ( $(mut $arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    };
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[get($route_path:tt)]
        $(#[$attr:meta])*
        pub async fn $name:ident( $(mut $arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        doc_comment! {
            concat!(
                $description, "\n"
            ),
            #[ doc = "### Example request:" ]
            #[ doc = "```bash" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[get($route_path)]
            $( #[$attr] )*
            pub async fn $name ( $(mut $arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    }
}

create_api_with_doc!(
    #[describe("List the simulations")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation")]
    #[get("/simulation")]
    pub async fn get_simulation() -> Result<SimulationJson, String> {
        match write_u64(&String::from("redis_key"), 36u64) {
            Ok(_) => (),
            Err(e) => return Err(format!("Could not write to redis DB: {}", e))
        };
        match read_u64(&String::from("redis_key")) {
            Ok(i) => Ok(Json(Simulation { model_id: i, simulation_id: 0, simulation_type: "Powerflow".to_string() })),
            Err(e) => Err(format!("Could not write to redis DB: {}", e))
        }
    }
);

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

use rocket::http::Status;

create_api_with_doc!(
    #[describe("Create a new simulation")]
    #[example("
        curl -X POST
             -H 'Accept: application/json'
             -F file=@/path/to/example.zip
             -F model_id=1
             -F allowed_file_types=application/zip http://localhost:8000/simulation"
    )]
    #[post("/simulation", format = "multipart/form-data", data = "<form>")]
    pub async fn post_simulation(mut form: Form < SimulationForm < '_ > > ) -> (Status, Json<Simulation>) {
        if let Some(name) = form.load_profile_data.name() {
            let tmp_dir = PathBuf::from("/tmp/");
            let save_name = tmp_dir.join(name);
            info!("Saving form to: {:?}", save_name);
            match form.load_profile_data.persist_to(save_name).await {
                Ok (_) => (Status::Accepted, Json(Simulation {
                    simulation_id: 0,
                    simulation_type: form.simulation_type.clone(),
                    model_id: form.model_id
                })),
                Err(e) => (Status::Conflict, Json(Simulation {
                    simulation_id: 0,
                    simulation_type: format!("Failed to store load profile data: {}", e),
                    model_id: 0
                }))
            }
        }
        else {
            (Status::BadRequest, Json(Simulation {
                simulation_id: 0,
                simulation_type: String::from("Bad request, no file name specified for load profile data."),
                model_id: 1
            }))
        }
    }
);

#[catch(422)]
pub async fn incomplete_form(form: &rocket::Request<'_>) -> String {
    info!("FORM: {}", form);
    "Incomplete form".to_string()
}

#[rocket::main]
#[doc = "The main entry point for Rocket" ]
async fn main() -> Result <(), rocket::Error> {
    rocket::build()
        .register("/", catchers![incomplete_form])
        .mount("/", routes![get_simulation, post_simulation])
        .launch()
        .await
}
