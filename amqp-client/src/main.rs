use log::{info};
use amqp_client::{Result, ConnectionFactory, ExchangeType};

const EXCHANGE: &str = "my-exchange";
const ROUTING_KEY: &str = "my.key";

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();

  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;
  let channel = connection.create_channel().await?;

  channel.declare_exchange(EXCHANGE.into(), ExchangeType::Direct, true, false, false, false,None).await?;
  let queue = channel.declare_queue("", false, false, false, false, None).await?;

  channel.bind(queue.clone(), EXCHANGE.into(), ROUTING_KEY.into()).await?;

  let mut queue_recv = channel.consume(queue.clone()).await?;
  tokio::spawn(async move {
    while let Some(frame) = queue_recv.recv().await {
      println!("Received a message: {:?}", String::from_utf8(frame.body.unwrap()));
    }
  });

  channel.publish(EXCHANGE.into(), ROUTING_KEY.into(), "Hello world!".as_bytes().to_vec()).await?;

  let mut s = String::new();
  println!("Waiting ...");
  std::io::stdin().read_line(&mut s).unwrap();

  info!("Channel closed ...");
  Ok(())
}

