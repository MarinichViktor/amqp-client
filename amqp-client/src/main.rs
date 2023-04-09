use log::{info};
use amqp_client::{Result, ConnectionFactory, Fields};
use amqp_client::protocol::types::{UShort, Short};

const EXCHANGE: &str = "my-exchange";
const ROUTING_KEY: &str = "my.key";
use paste::paste;


#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();
  // let start = Start {
  //   ver_min: 1,
  //   ver_maj: 2
  // };
  // let bytes = vec![0, 3_u8, 1, 1_u8,0, 3_u8, 1, 1_u8];
  // let start = Start::from_raw_repr(&bytes[..]);
  // println!("Start {:?}", start);
  //
  // let into = start.to_raw_repr();
  // println!("Start into {:?}", into);
  // return Ok(());
  //
  let connection_uri = "amqp://user:password@localhost:5672/my_vhost";
  let mut connection = ConnectionFactory::create(connection_uri).await?;
  // let channel = connection.create_channel().await?;
  //
  // channel.declare_exchange(EXCHANGE, ExchangeType::Direct, true, false, false, false,None).await?;
  // let queue = channel.declare_queue("", false, false, false, false, None).await?;
  //
  // channel.bind(&queue, EXCHANGE, ROUTING_KEY).await?;
  //
  // let mut queue_recv = channel.consume(&queue).await?;
  // tokio::spawn(async move {
  //   while let Some(frame) = queue_recv.recv().await {
  //     println!("Received a message: {:?}", String::from_utf8(frame.body.unwrap()));
  //     println!("Received a message: {:?}", frame.prop_fields);
  //   }
  // });
  //
  // let mut fields = Fields::new();
  // fields.content_type = Some("text/plain".into());
  // fields.reply_to = Some("abcefg".into());
  // channel.publish(EXCHANGE, ROUTING_KEY, "Hello world!".as_bytes().to_vec(), Some(fields)).await?;

  let mut s = String::new();
  println!("Waiting ...");
  std::io::stdin().read_line(&mut s).unwrap();

  info!("Channel closed ...");
  Ok(())
}

