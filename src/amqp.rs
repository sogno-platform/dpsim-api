use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};
use log::info;

pub async fn publish(simulation: &super::Simulation) -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default().with_default_executor(8),
    )
    .await?;

    info!("CONNECTED");

    let channel_a = conn.create_channel().await?;

    let queue = channel_a
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await?;

    info!("Declared queue {:?}", queue);

    let payload = b"Hello world!";
        let confirm = channel_a
            .basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                payload.to_vec(),
                BasicProperties::default(),
            )
            .await?
            .await?;
        assert_eq!(confirm, Confirmation::NotRequested);
    Ok(())
}
