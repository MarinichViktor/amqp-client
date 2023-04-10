# RabbitMQ client
For learning/research purpose.

## Example:
1. Create connection, channel, exchange

```rust
  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;

  let channel = connection.create_channel().await?;
  channel.declare_exchange("my-exchange", ExchangeType::Direct, true, false, false, false,None).await?;

  let queue = channel.declare_queue("", false, false, false, false, None).await?;
  channel.bind(&queue, "my-exchange", "my.key").await?;
```
2. Specify message content, properties, and publish to the server
```rust
  let mut properties = MessageProperties::new();
  properties.content_type = Some("text/plain".into());
  properties.timestamp = Some(
   SystemTime::now()
     .duration_since(SystemTime::UNIX_EPOCH)
     .unwrap()
  );

  channel.publish("my-exchange", "my.key", Message {
    properties,
    content: "Hello world!".as_bytes().to_vec()
  }).await?;
  ...
}
```

3. Create queue consumer
```rust
  let mut consumer_rx = channel.consume(&queue).await?;
  tokio::spawn(async move {
    while let Some(message) = consumer_rx.recv().await {
      println!("Message:\n\t {}", String::from_utf8(message.content).unwrap());
      println!("Properties:\n\t {:?}", message.properties);
    }
  });
```
