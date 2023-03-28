use log::{info};
use amqp_client::{Result, ConnectionFactory, ExchangeType};

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();

  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;
  info!("Connected to the server");
  let channel = connection.create_channel().await?;
  info!("Created channel");
  channel.declare_exchange("new_exchange".into(), ExchangeType::Direct, true, false, false, false,None).await?;
  info!("Exchange declared");
  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  info!("Queue declared {}", queue);
  channel.bind(queue, "exchange".into(), "some.path".into()).await?;

  // // Binding queue to the exchange and starting consumer
  // let routing_key = String::from("foo.bar");
  // chan.bind(
  //   queue.clone(),
  //   exchange.clone(),
  //   routing_key.clone()
  // )?;
  //
  // let queue_recv = chan.consume(queue.clone())?;
  // thread::spawn(move || {
  //   for frame in queue_recv {
  //     let body = frame.content_body.unwrap();
  //     println!("Received body frame: {:?}", String::from_utf8(body));
  //   }
  // });

  let mut s = String::new();
  println!("Waiting ...");
  std::io::stdin().read_line(&mut s).unwrap();

  info!("Channel closed ...");
  Ok(())
}

