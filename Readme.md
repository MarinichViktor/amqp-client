# RabbitMQ client
#### For learning/research purpose.
The main goal of this project is to implement a subset of the AMQP-0.9.1 protocol. At the same time, a client should provide the bulk of the protocol functions.
## Example:


```rust
  // Create connection and channel
  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;
  let channel = connection.create_channel().await?;

  // Declare exchange, bind it to the queue
  channel.declare_exchange( "my-exchange", ExchangeType::Direct, true, false, false,
    false, None).await?;

  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  channel.bind(&queue, "my-exchange", "my.key").await?;

  // Subscribe to the queue messages
  let mut consumer_rx = channel.consume(&queue).await?;
  tokio::spawn(async move {
    while let Some(message) = consumer_rx.recv().await {
      println!("Message:\n\t{}", String::from_utf8(message.get_body().into()).unwrap());
      println!("Properties:\n\t{:?}", message.get_properties());
      message.ack(false).unwrap();
    }
  });


  let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap();

  // Publish message
  let mut properties = MessageProperties::new();
  properties.timestamp = Some(timestamp);
  properties.content_type = Some("text/plain".into());

  channel.publish("my-exchange", "my.key", "Hello world!".into(), properties).await?;
```
