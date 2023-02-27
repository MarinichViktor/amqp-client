# Rabbitmq client (AMQP-0.9.1 client)

Learning project with the main goal to build the rabbitmq client via implementation bulk of the AMQP protocol.


How to run client:
```rust
  let mut connection = Connection::new(
    "host".to_string(),
    5672,
    "user".to_string(),
    "password".to_string(),
    "vhost".to_string()
  );
  // connect
  connection.connect()?;

  // create channel
  let chan = connection.create_channel()?;

  // declare exchange
  chan.declare_exchange(|builder| {
    builder.name("my_exchange".to_string());
  })?;

  // declare queue
  chan.declare_queue(|builder| {
    builder.name(String::from("my_queue"));
  })?;

  // bind queue and exchange
  chan.bind("my_queue", "my_exchange", "some_key")?;
  chan.consume("q123".to_string(), "cons1".to_string())?;

  //TDB
```
To be done:
- [x] Connection
- [x] Channel
- [ ] Exchange
- [ ] Queue
- [ ] Basic
- [ ] Tx
