use std::collections::HashMap;
use log::{info};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use crate::building_blocks::{Command, CommandPayload};
use crate::protocol::types::{ChannelId, Long, ShortStr, PropTable};
use crate::{invoke_sync_method, invoke_command_async, Result, unwrap_frame_variant, MessageProperties};
use crate::api::exchange::{ExchangeDeclareOptsBuilder, ExchangeType};
use crate::api::queue::QueueDeclareOptsBuilder;
use crate::protocol::message::{Message};
use crate::protocol::frame::{FrameEnvelope, Frame, BasicConsume, BasicPublish, ChannelOpen,
                             ContentBody, ContentHeader, ExchangeDeclare, QueueBind, QueueDeclare,
                             QueueUnbind};

pub struct AmqChannel {
  pub id: ChannelId,
  outgoing_tx: UnboundedSender<FrameEnvelope>,
  command_tx: UnboundedSender<Command>
}

impl AmqChannel {
  pub async fn open(
    id: ChannelId,
    outgoing_tx: UnboundedSender<FrameEnvelope>,
    incoming_rx: UnboundedReceiver<FrameEnvelope>,
    command_tx: UnboundedSender<Command>,
  ) -> Result<Self> {
    let open_method = ChannelOpen { reserved1: ShortStr("".into()) }.into_frame();
    let _frame = invoke_sync_method!(id, command_tx, outgoing_tx, open_method).await?;
    let channel = Self {
      id,
      outgoing_tx,
      command_tx
    };

    // channel.spawn_incoming_msg_handler(incoming_rx);

    Ok(channel)
  }

  // todo: do we need this?
  fn spawn_incoming_msg_handler(&self, mut incoming_rx: UnboundedReceiver<FrameEnvelope>) {
    tokio::spawn(async move {
      while let Some((_, frame)) = incoming_rx.recv().await {
        match frame {
          _ => {
            todo!("Implement handler")
          }
        }
      }
    });
  }

  pub async fn declare_exchange(
    &self,
    name: &str,
    ty: ExchangeType,
    durable: bool,
    passive: bool,
    auto_delete: bool,
    internal: bool,
    props: Option<PropTable>
  ) -> Result<()>
  {
    self.declare_exchange_with_builder(|builder| {
      builder.name(name.into());
      builder.ty(ty);
      builder.durable(durable);
      builder.passive(passive);
      builder.auto_delete(auto_delete);
      builder.internal(internal);
      builder.props(props.unwrap_or_else(|| HashMap::new()))
    }).await
  }

  pub async fn declare_exchange_with_builder<F>(&self, configure: F) -> Result<()>
    where F: FnOnce(&mut ExchangeDeclareOptsBuilder) -> ()
  {
    info!("declare exchange");
    let mut builder = ExchangeDeclareOptsBuilder::new();
    configure(&mut builder);
    let method = ExchangeDeclare::from(builder.build());
    let _frame = invoke_sync_method!(self.id, self.command_tx, self.outgoing_tx, method.into_frame()).await?;
    info!("declared exchange");

    Ok(())
  }

  pub async fn declare_queue(
    &self,
    name: &str,
    durable: bool,
    passive: bool,
    auto_delete: bool,
    exclusive: bool,
    props: Option<PropTable>
  ) -> Result<String> {
    self.declare_queue_with_builder(move |builder| {
      builder.name(name.to_string());
      builder.durable(durable);
      builder.passive(passive);
      builder.auto_delete(auto_delete);
      builder.exclusive(exclusive);
      builder.no_wait(false);
      builder.props(props.unwrap_or_else(|| PropTable::new()));
    }).await
  }
  async fn invoke_sync_method(&self, frame: Frame) -> Result<Frame> {
    Ok(invoke_sync_method!(self.id, self.command_tx, self.outgoing_tx, frame).await?)
  }

  pub async fn declare_queue_with_builder<F>(&self, configure: F) -> Result<String>
    where F: FnOnce(&mut QueueDeclareOptsBuilder) -> ()
  {
    info!("declare queue");
    let mut opts = QueueDeclareOptsBuilder::new();

    configure(&mut opts);

    let method = QueueDeclare::from(opts.build());
    let frame = self.invoke_sync_method(method.into_frame()).await?;
    let declare_ok = unwrap_frame_variant!(frame, QueueDeclareOk);
    info!("declared queue {}", &declare_ok.name.0);
    // todo: into impl
    Ok(declare_ok.name.0)
  }

  pub async fn bind(&self, queue_name: &str, exchange_name: &str, routing_key: &str) -> Result<()> {
    info!("bind queue: {} to: exchange {} with key: {}", queue_name.clone(), exchange_name.clone(), routing_key.clone());
    let method = QueueBind {
      reserved1: 0,
      queue: queue_name.into(),
      exchange: exchange_name.into(),
      routing_key: routing_key.into(),
      no_wait: 0,
      table: HashMap::new()
    };

    let frame = self.invoke_sync_method(method.into_frame()).await?;
    let _bind_ok = unwrap_frame_variant!(frame, QueueBindOk);
    info!("queue bound");

    Ok(())
  }

  pub async fn unbind(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
    let method = QueueUnbind {
      reserved1: 0,
      queue: queue.into(),
      exchange: exchange.into(),
      routing_key: routing_key.into(),
      table: HashMap::new()
    };

    let frame = self.invoke_sync_method(method.into_frame()).await?;
    let _bind_ok = unwrap_frame_variant!(frame, QueueUnbindOk);
    info!("queue bound");

    Ok(())
  }

  pub async fn consume(&self, queue: &str) -> Result<UnboundedReceiver<Message>> {
    info!("consuming queue: {}", queue.clone());
    let method = BasicConsume {
      reserved1: 0,
      queue: queue.into(),
      tag: "".into(),
      flags: 0,
      props: HashMap::new()
    };

    let frame = self.invoke_sync_method(method.into_frame()).await?;
    let consume_ok = unwrap_frame_variant!(frame, BasicConsumeOk);

    let (consumer_tx, consumer_rx) = mpsc::unbounded_channel();
    invoke_command_async!(self.command_tx, CommandPayload::RegisterConsumer(self.id, consume_ok.tag.0.clone(), consumer_tx));
    info!("consume ok with tag: {}", consume_ok.tag.0);

    Ok(consumer_rx)
  }

  pub async fn publish(&self, exchange: &str, routing_key: &str, body: Vec<u8>, properties: MessageProperties) -> Result<()> {

    info!("Publishing message");
    let method = BasicPublish {
      reserved1: 0,
      exchange: exchange.into(),
      routing_key: routing_key.into(),
      flags: 0,
    };
    let header = ContentHeader {
      class_id: 60,
      body_len: body.len() as Long,
      prop_list: properties,
    };
    let body = ContentBody(body);
    self.outgoing_tx.send((self.id, method.into_frame())).unwrap();
    self.outgoing_tx.send((self.id, header.into_frame())).unwrap();
    self.outgoing_tx.send((self.id, body.into_frame())).unwrap();


    info!("Message was published");

    Ok(())
  }

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
}
