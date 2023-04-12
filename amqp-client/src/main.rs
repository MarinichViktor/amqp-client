use std::time::{Duration, SystemTime};
use amqp_client::{Result, ConnectionFactory, ExchangeType, MessageProperties};


#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();

  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;

  let channel = connection.create_channel().await?;
  channel.declare_exchange("my-exchange", ExchangeType::Direct, true, false, false, false,None).await?;

  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  channel.bind(&queue, "my-exchange", "my.key").await?;

  let mut consumer_rx = channel.consume(&queue).await?;
  tokio::spawn(async move {
    while let Some(message) = consumer_rx.recv().await {
      println!("Message:\n\t{}", String::from_utf8(message.get_body().into()).unwrap());
      println!("Properties:\n\t{:?}", message.get_properties());
      message.ack(false).unwrap();
    }
  });

  let mut properties = MessageProperties::new();
  let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap();
  properties.timestamp = Some(timestamp);
  properties.content_type = Some("text/plain".into());
  channel.publish("my-exchange", "my.key", "Hello world!".into(), properties).await?;

  tokio::time::sleep(Duration::from_secs(2)).await;
  connection.close().await?;
  tokio::time::sleep(Duration::from_secs(5)).await;

  Ok(())
}

