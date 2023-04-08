// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
// use crate::{ExchangeType, Fields, Result};
// use log::{debug, info};
// use crate::connection::{FrameTransmitter};
// use tokio::sync::{oneshot, mpsc};
// use tokio::sync::mpsc::UnboundedReceiver;
// use crate::protocol::basic::methods::{Deliver};
// use crate::channel::methods::{Open, OpenOk};
// use crate::protocol::frame2::{Frame2, RawFrame};
// use amqp_protocol::types::{AmqpMethodArgs, Table};
// use crate::exchange::ExchangeDeclareOptsBuilder;
// use crate::protocol::writer::FrameWriter;
// use crate::queue::QueueDeclareOptsBuilder;
//
// pub mod methods;
// pub mod constants;
//
// pub struct AmqChannel {
//   pub id: i16,
//   con_writer: Arc<tokio::sync::Mutex<FrameWriter>>,
//   channel_notifier: Option<mpsc::Receiver<RawFrame>>,
//   active: bool,
//   consumers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<RawFrame>>>>,
//   sync_waiter_queue: Arc<Mutex<Vec<oneshot::Sender<RawFrame>>>>
// }
//
// impl AmqChannel {
//   pub(crate) fn new(id: i16, connection_notifier: Arc<tokio::sync::Mutex<FrameWriter>>, channel_notifier: mpsc::Receiver<RawFrame>) -> Self {
//     Self {
//       id,
//       con_writer: connection_notifier,
//       channel_notifier: Some(channel_notifier),
//       active: true,
//       consumers: Arc::new(Mutex::new(Default::default())),
//       sync_waiter_queue: Arc::new(Mutex::new(vec![]))
//     }
//   }
//
//   pub async fn open(&mut self) -> Result<OpenOk> {
//     let sync_waiter_queue = self.sync_waiter_queue.clone();
//     let mut inner_rx = self.channel_notifier.take().unwrap();
//     let consumers = self.consumers.clone();
//
//     info!("[Channel] start incoming listener");
//     tokio::spawn(async move {
//       use crate::protocol::{basic};
//
//       loop {
//         if let Some(frame) = inner_rx.recv().await {
//           match (frame.cid, frame.mid) {
//             (60, basic::constants::METHOD_DELIVER) => {
//               let payload: Deliver = frame.args.clone().try_into().unwrap();
//               let consumers = consumers.lock().unwrap();
//               let handler = consumers.get(&payload.consumer_tag).unwrap();
//               handler.send(frame).unwrap();
//             },
//             _ => {
//               let mut sync_waiter_queue = sync_waiter_queue.lock().unwrap();
//               sync_waiter_queue.pop().unwrap().send(frame).unwrap();
//             }
//           }
//         } else {
//           // todo: review action
//           panic!("Channel receiver channel");
//         }
//       }
//     });
//
//     let method = Open::default();
//     let response = self.invoke_sync_method(method).await?;
//     Ok(response.args.try_into()?)
//   }
//
//   pub async fn declare_exchange(
//     &self,
//     name: &str,
//     ty: ExchangeType,
//     durable: bool,
//     passive: bool,
//     auto_delete: bool,
//     internal: bool,
//     props: Option<Table>
//   ) -> Result<()>
//   {
//     self.declare_exchange_with_builder(|builder| {
//       builder.name(name.into());
//       builder.ty(ty);
//       builder.durable(durable);
//       builder.passive(passive);
//       builder.auto_delete(auto_delete);
//       builder.internal(internal);
//       builder.props(props.unwrap_or_else(|| HashMap::new()))
//     }).await
//   }
//
//   pub async fn declare_exchange_with_builder<F>(&self, configure: F) -> Result<()>
//     where F: FnOnce(&mut ExchangeDeclareOptsBuilder) -> ()
//   {
//     use crate::exchange::methods::{Declare};
//
//     debug!("Declare exchange");
//     let mut builder = ExchangeDeclareOptsBuilder::new();
//     configure(&mut builder);
//     self.invoke_sync_method(Declare::from(builder.build())).await?;
//
//     Ok(())
//   }
//
//   pub async fn declare_queue(
//     &self,
//     name: &str,
//     durable: bool,
//     passive: bool,
//     auto_delete: bool,
//     exclusive: bool,
//     props: Option<Table>
//   ) -> Result<String> {
//     self.declare_queue_with_builder(move |builder| {
//       builder.name(name.to_string());
//       builder.durable(durable);
//       builder.passive(passive);
//       builder.auto_delete(auto_delete);
//       builder.exclusive(exclusive);
//       builder.no_wait(false);
//       builder.props(props.unwrap_or_else(|| Table::new()));
//     }).await
//   }
//
//   pub async fn declare_queue_with_builder<F>(&self, configure: F) -> Result<String>
//     where F: FnOnce(&mut QueueDeclareOptsBuilder) -> ()
//   {
//     use crate::queue::methods::{Declare, DeclareOk};
//
//     debug!("Declare queue");
//     let mut opts = QueueDeclareOptsBuilder::new();
//
//     configure(&mut opts);
//
//     let response = self.invoke_sync_method(Declare::from(opts.build())).await?;
//     let payload: DeclareOk = response.args.try_into()?;
//
//     Ok(payload.name)
//   }
//
//   pub async fn bind(&self, queue_name: &str, exchange_name: &str, routing_key: &str) -> Result<()> {
//     use crate::queue::methods::Bind;
//
//     debug!("Bind queue: {} to: exchange {} with key: {}", queue_name.clone(), exchange_name.clone(), routing_key.clone());
//     let payload = Bind {
//       reserved1: 0,
//       queue_name: queue_name.into(),
//       exchange_name: exchange_name.into(),
//       routing_key: routing_key.into(),
//       no_wait: 0,
//       table: HashMap::new()
//     };
//     self.invoke_sync_method(payload).await?;
//
//     Ok(())
//   }
//
//   pub async fn unbind(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
//     use crate::queue::methods::Unbind;
//
//     let payload = Unbind {
//       reserved1: 0,
//       queue_name: queue.to_string(),
//       exchange_name: exchange.to_string(),
//       routing_key: routing_key.to_string(),
//       table: HashMap::new()
//     };
//
//     self.invoke_sync_method(payload).await?;
//
//     Ok(())
//   }
//
//   pub async fn flow(&self, active: bool) -> Result<()> {
//     use self::methods::Flow;
//
//     debug!("Invoking channel {} Flow", self.id);
//     if self.active == active {
//       return Ok(())
//     }
//
//     self.invoke_sync_method(Flow { active: active as u8 }).await?;
//     // self.active = active;
//
//     Ok(())
//   }
//
//   pub async fn consume(&self, queue: &str) -> Result<UnboundedReceiver<RawFrame>> {
//     use crate::protocol::basic::methods::{Consume,ConsumeOk};
//
//     info!("Consuming queue: {}", queue.clone());
//     let payload = Consume {
//       reserved1: 0,
//       queue: queue.into(),
//       tag: String::from(""),
//       flags: 0,
//       table: HashMap::new()
//     };
//
//     let response = self.invoke_sync_method(payload).await?;
//     let payload: ConsumeOk = response.args.try_into()?;
//
//     info!("Consume ok with tag: {}", payload.tag.clone());
//     let (tx, rx) = mpsc::unbounded_channel();
//     self.consumers.lock().unwrap().insert(payload.tag, tx);
//
//     Ok(rx)
//   }
//
//   pub async fn publish(&self, exchange: &str, routing_key: &str, payload: Vec<u8>, fields: Option<Fields>) -> Result<()> {
//     use crate::protocol::basic::methods::{Publish};
//
//     info!("Publishing message");
//     let method = Publish {
//       reserved1: 0,
//       exchange: exchange.into(),
//       routing_key: routing_key.into(),
//       flags: 0,
//     };
//     let frame = Frame2 {
//       ch: self.id,
//       args: method,
//       prop_fields: fields,
//       body: Some(payload)
//     };
//
//     let mut writer = self.con_writer.lock().await;
//     writer.send_raw_frame(frame.into()).await?;
//     info!("Message was published");
//
//     Ok(())
//   }
//
//   pub async fn close(&self) -> Result<()> {
//     use crate::channel::methods::Close;
//
//     debug!("Closing channel {}", self.id);
//     self.invoke_sync_method(Close {
//       reply_code: 200,
//       reply_text: "Closed".to_string(),
//       class_id: 0,
//       method_id: 0,
//     }).await?;
//
//     Ok(())
//   }
//
//   async fn invoke_sync_method<T: AmqpMethodArgs>(&self, args: T) -> Result<RawFrame> {
//     let (tx, rx) = oneshot::channel::<RawFrame>();
//     let sync_waiter_queue = self.sync_waiter_queue.clone();
//     sync_waiter_queue.lock().unwrap().push(tx);
//
//     let request: Frame2<T> = Frame2::new(self.id, args);
//     {
//       let mut writer = self.con_writer.lock().await;
//       writer.send_raw_frame(request.into()).await?;
//     }
//
//     Ok(rx.await?)
//   }
// }
