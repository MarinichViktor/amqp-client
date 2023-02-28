# Rabbitmq client (AMQP-0.9.1 client)

Learning project with the main goal to build the rabbitmq client via implementation bulk of the AMQP protocol.


Usage example:
1. Establish connection
```rust
let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
let mut connection = Connection::from_uri(connection_uri)?;
connection.connect()?;
```

2. Create channel, queue and exchange
```rust
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
```
3. Binding queue to the exchange and start a consumer
```rust
let routing_key = String::from("foo.bar");
channel.bind(
  queue.clone(),
  exchange.clone(),
  routing_key.clone()
)?;

//TDB
```
To be done:
- [x] Connection
- [x] Channel
- [X] Exchange
- [x] Queue
- [ ] Basic
- [ ] Tx
