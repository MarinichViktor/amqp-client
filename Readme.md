# Rabbitmq client (AMQP-0.9.1 client)

## Goals
The main goal of this project is to build as simple as possible but the working implementation of the RabbitMQ client
via the implementation bulk of the AMQP protocol. At the same time, this project serves as my pet project for RUST lang learning.

## Current status
At this stage, it's possible to establish a connection with a server and create a new channel. Queues and Exchanges could be
declared through a channel, and bonded with a routing key. Also, users could start a consumer for the queue.
TODO: implement message publishing.

Example:
```rust
let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
let mut connection = Connection::from_uri(connection_uri)?;
connection.connect()?;

// Create channel, queue and exchange
let channel = connection.create_channel()?;
let exchange = String::from("exch1");
channel.exchange_declare(
  exchange.clone(),
  ExchangeType::Direct,
  true,
  false,
  false,
  false,
  None
)?;

let queue = channel.queue_declare(
  "",
  false,
  false,
  false,
  false,
  None
)?;
//Binding queue to the exchange and start a consumer

let routing_key = String::from("foo.bar");
channel.bind(
  queue.clone(),
  exchange.clone(),
  routing_key.clone()
)?;

let queue_recv = channel.consume(queue.clone())?;
thread::spawn(move || {
  for frame in queue_recv {
    let body = frame.content_body.unwrap();
    // Assuming that body is just a raw string
    println!("Received body frame: {:?}", String::from_utf8(body));
  }
});

//TDB
```
To be done:
- [x] Connection
- [x] Channel
- [X] Exchange
- [x] Queue
- [ ] Basic
- [ ] Tx
