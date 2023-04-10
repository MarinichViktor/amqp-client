# RabbitMQ client
For learning/research purpose.

## To be done:
- [x] Connection
- [x] Channel
- [X] Exchange
- [x] Queue
- [ ] Basic (In progress)
- [ ] Tx

## Usage:

Create connection and channel
```rust
const EXCHANGE: &str = "my-exchange";
const ROUTING_KEY: &str = "my.key";

let mut connection = ConnectionFactory::create("amqp://user:password@localhost:5672/my_vhost").await?;
let channel = connection.create_channel().await?;
```


Declare exchange, queue and bind them
```rust
channel.declare_exchange(EXCHANGE, ExchangeType::Direct, true, false, false, false,None).await?;
let queue = channel.declare_queue("", false, false, false, false, None).await?;
channel.bind(&queue, EXCHANGE, ROUTING_KEY).await?;
```

Start queue consumer
```rust
let mut consumer_rx = channel.consume(queue.clone()).await?;
tokio::spawn(async move {
    while let Some(delivery) = consumer_rx.recv().await {
      // We assume that message is a valid utf8 string
      println!("Received a message: {:?}", String::from_utf8(delivery.body.unwrap()));
    }
});
```

Publish message to the queue
```rust
channel.publish(EXCHANGE, ROUTING_KEY, "Hello world!".as_bytes().to_vec()).await?;
```

