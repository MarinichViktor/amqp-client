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
  connection.open().unwrap();

  let chan = connection.create_channel().unwrap();
  chan.close().unwrap();

  sleep(Duration::from_secs(5));
}

