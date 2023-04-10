use std::time::{Duration, SystemTime};
use log::{info};
use amqp_client::{Result, ConnectionFactory};
use amqp_client::api::basic::fields::MessageProperties;

use amqp_client::api::exchange::ExchangeType;
use amqp_client::internal::channel::Message;


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
      println!("Message:\n\t{}", String::from_utf8(message.content).unwrap());
      println!("Properties:\n\t{:?}", message.properties);
      // todo: implement
      // channel.ack()/nack();
    }
  });

  let mut properties = MessageProperties::new();
  properties.content_type = Some("text/plain".into());
  properties.timestamp = Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap());

  let message = Message {
    properties,
    content: "Hello world!".as_bytes().to_vec()
  };

  channel.publish("my-exchange", "my.key", message).await?;

  println!("Waiting ...");
  let mut s = String::new();
  std::io::stdin().read_line(&mut s).unwrap();
  Ok(())
}

