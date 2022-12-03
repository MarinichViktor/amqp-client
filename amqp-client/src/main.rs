use std::thread::{sleep};
use std::time::Duration;
use log::{info};
use amqp_client::protocol::connection::AmqConnection;
use amqp_client::response;

fn main() -> response::Result<()> {
  env_logger::init();
  // let test_map = HashMap::new();
  // return Ok(());

  let mut connection = AmqConnection::new(
    "localhost".to_string(),
    5672,
    "user".to_string(),
    "password".to_string(),
    "my_vhost".to_string()
  );
  info!("Connection connect ...");
  connection.connect()?;

  let chan = connection.create_channel()?;
  chan.close()?;
  sleep(Duration::from_secs(5));

  // match connection.connect() {
  //     Err(e) => {
  //         for x in e.chain() {
  //             info!("{}", x)
  //         }
  //     },
  //     _ => {}
  // }

  info!("Connection connect finished");

  Ok(())
}

