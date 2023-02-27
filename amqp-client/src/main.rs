use std::io::stdin;
use std::thread::{sleep};
use std::time::Duration;
use log::{info};
use amqp_client::{Connection};

fn main() {
  env_logger::init();
  let mut connection = Connection::new(
    "localhost".to_string(),
    5672,
    "user".to_string(),
    "password".to_string(),
    "my_vhost".to_string()
  );

  info!("Connection connect ...");
  connection.connect().unwrap();
  info!("Connection finished ...");

  info!("Declaring exchange...");
  let chan = connection.create_channel().unwrap();
  let _exchange = chan.declare_exchange(|builder| {
    builder.name("my_awesome_exchange2".to_string());
  }).unwrap();

  info!("Declaring queue...");
  let _queue = chan.declare_queue(|builder| {
    builder.name(String::from("q123"));
  });
  info!("Binding queue...");
  chan.bind("q123", "my_awesome_exchange2", "go_here").unwrap();
  info!("Starting consume...");
  chan.consume("q123".to_string(), "cons1".to_string()).unwrap();
  info!("Started consume...");
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
}
