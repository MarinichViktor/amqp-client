use std::collections::HashMap;
use std::thread::{sleep, Thread};
use std::time::Duration;
use log::{info};
use amqp_client::connection::Connection;
use amqp_client::response;

fn main() -> response::Result<()> {
    env_logger::init();
    // let test_map = HashMap::new();
    // return Ok(());

    let mut connection = Connection::new("localhost".to_string(), 5672, "user".to_string(), "password".to_string());
    info!("Connection connect ...");
    connection.connect()?;

    let chan = connection.create_channel()?;
    sleep(Duration::from_secs(2));
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

