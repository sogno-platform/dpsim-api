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
use crate::routes::AMQPSimulation;
use rocket::serde::json::json;
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
