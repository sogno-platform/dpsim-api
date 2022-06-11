#[cfg(not(test))]
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};
#[cfg(not(test))]
use log::info;
#[cfg(test)]
use lapin::{
    Result,
};
use crate::routes::{Simulation, SimulationType};
use rocket::serde::json::{json, Json};
use serde::{ Serialize, Deserialize };
use schemars::JsonSchema;
#[cfg(test)]
pub async fn publish(bytes: Vec<u8>) -> Result<()> {
    println!("AMQPSimulation: {:?}", bytes);
    Ok(())
}
#[cfg(not(test))]
pub async fn publish(bytes: Vec<u8>) -> Result<()> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://rabbitmq:5672/%2f".into());

    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default().with_default_executor(8),
    )
    .await?;

    info!("CONNECTED TO AQMP SERVER");

    let channel_a = conn.create_channel().await?;

    let queue = channel_a
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await?;

    info!("Declared queue {:?}", queue);

    let confirm = channel_a
        .basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            bytes,
            BasicProperties::default(),
        )
        .await?
        .await?;
    assert_eq!(confirm, Confirmation::NotRequested);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[doc = "Struct for encapsulation Simulation details"]
pub struct AMQPSimulation {
    error: String,
    load_profile_url:  String,
    model_url:         String,
    simulation_id:     u64,
    simulation_type:   SimulationType,
    results_file:      String
}

impl AMQPSimulation {
    pub fn from_simulation(sim: &Json<Simulation>, model_url: String, load_profile_url: String) -> AMQPSimulation {
        AMQPSimulation {
            error:            "".into(),
            load_profile_url: load_profile_url,
            model_url:        model_url,
            simulation_id:    sim.simulation_id,
            simulation_type:  sim.simulation_type,
            results_file:     sim.results_id.clone(),
        }
    }
}

pub async fn request_simulation(_simulation: &AMQPSimulation) -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let message_as_jsonvalue = json!({
      "model" : {
        "type" : "url-list",
        "url" : [ _simulation.model_url ]
      },
      "parameters": {
        "results_file": _simulation.results_file,
        "executable": "SLEW_Shmem_CIGRE_MV_PowerFlow",
        "name": "SLEW_Shmem_CIGRE_MV_PowerFlow",
        "timestep": 0.1,
        "duration": 20
      }
    });
    let message = serde_json::to_vec(&message_as_jsonvalue).unwrap();

    publish(message).await?;

    Ok(())
}
