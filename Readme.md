# RabbitMQ client 
For learning/research purposes.

## Goals
The main goal of this project is to build as simple as possible but the working implementation of the RabbitMQ client
via the implementation bulk of the AMQP protocol. At the same time, this project serves as my pet project for RUST lang learning.

## Current status
At this stage, it's possible to establish a connection with a server and create a new channel. Queues and Exchanges could be
declared through a channel, and bonded with a routing key. Also, users could start a consumer for the queue.
TODO: implement message publishing.

## Example:
```rust
const EXCHANGE: &str = "my-exchange";
const ROUTING_KEY: &str = "my.key";

#[tokio::main]
async fn main() -> Result<()> {
  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";

  // Create connection and channel
  let mut connection = ConnectionFactory::create(connection_uri).await?;
  let channel = connection.create_channel().await?;

  // Declare exchange, queue and bind them
  channel.declare_exchange(EXCHANGE.into(), ExchangeType::Direct, true, false, false, false,None).await?;
  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  channel.bind(queue.clone(), EXCHANGE.into(), ROUTING_KEY.into()).await?;

  let mut consumer_rx = channel.consume(queue.clone()).await?;
  tokio::spawn(async move {
    while let Some(delivery) = consumer_rx.recv().await {
      // We assume that message is a valid utf8 string
      println!("Received a message: {:?}", String::from_utf8(delivery.body.unwrap()));
    }
  });

  channel.publish(EXCHANGE.into(), ROUTING_KEY.into(), "Hello world!".as_bytes().to_vec()).await?;

  Ok(())
}
```
## To be done:
- [x] Connection
- [x] Channel
- [X] Exchange
- [x] Queue
- [ ] Basic
- [ ] Tx
