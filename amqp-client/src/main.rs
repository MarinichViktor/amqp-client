use std::thread;
use log::{info};
use amqp_client::{Result, Connection, ExchangeType};

fn main() -> Result<()> {
  env_logger::init();
  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";

  // Establish connection
  let mut connection = Connection::from_uri(connection_uri)?;
  connection.connect()?;

  // Create channel, queue and exchange
  let chan = connection.create_channel()?;
  let exchange = String::from("exch1");
  chan.exchange_declare(
    exchange.clone(),
    ExchangeType::Direct,
    true,
    false,
    false,
    false,
    None
  )?;

  let queue = chan.queue_declare(
    "",
    false,
    false,
    false,
    false,
    None
  )?;

  // Binding queue to the exchange and starting consumer
  let routing_key = String::from("foo.bar");
  chan.bind(
    queue.clone(),
    exchange.clone(),
    routing_key.clone()
  )?;

  let queue_recv = chan.consume(queue.clone())?;
  thread::spawn(move || {
    for frame in queue_recv {
      let body = frame.content_body.unwrap();
      println!("Received body frame: {:?}", String::from_utf8(body));
    }
  });

  let mut s = String::new();
  println!("Waiting ...");
  std::io::stdin().read_line(&mut s).unwrap();
  // sleep(Duration::from_secs(15));
  // info!("Unbinding queue...");
  // chan.unbind("q123", "my_awesome_exchange2", "go_here").unwrap();
  // info!("Unbinding queue...");

  // sleep(Duration::from_secs(15));
  chan.close().unwrap();
  info!("Channel closed ...");
  Ok(())
}

async fn main_proto() -> Result<()> {
  env_logger::init();
  // let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let connection = ConnectionFactory::create("amqp://user:password@localhost:5672/my_vhost").await?;
  let channel = connection.declare_channel().await?;
  let exchange = channel.declare_exchange("exch1", ExchangeType::Direct, true, false, false, None).await?;
  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  channel.bind(queue, exchange, "some.path").await?;

  let queue_stream = channel.start_consumer(queue).await?;

  thread::spawn(move || {
    for frame in queue_stream {
      let body = frame.content_body.unwrap();
      println!("Received body frame: {:?}", String::from_utf8(body));
    }
  });

  let mut s = String::new();
  println!("Waiting ...");
  std::io::stdin().read_line(&mut s).unwrap();

  channel.close().unwrap();
  info!("Channel closed ...");
  Ok(())
}
