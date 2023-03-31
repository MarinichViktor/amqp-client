use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::{ExchangeType, Fields, Result};
use log::{debug, info};
use crate::connection::{FrameTransmitter};
use tokio::sync::{oneshot, mpsc};
use tokio::sync::mpsc::UnboundedReceiver;
use crate::protocol::basic::methods::{Deliver};
use crate::channel::methods::{Open, OpenOk};
use crate::protocol::frame2::{Frame2, RawFrame};
use amqp_protocol::types::{AmqpMethodArgs, Table};
use crate::exchange::ExchangeDeclareOptsBuilder;
use crate::queue::QueueDeclareOptsBuilder;

pub mod methods;
pub mod constants;

pub struct AmqChannel {
  pub id: i16,
  connection_notifier: Mutex<FrameTransmitter>,
  channel_notifier: Option<mpsc::Receiver<RawFrame>>,
  active: bool,
  consumers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<RawFrame>>>>,
  sync_waiter_queue: Arc<Mutex<Vec<oneshot::Sender<RawFrame>>>>
}

impl AmqChannel {
  pub(crate) fn new(id: i16, connection_notifier: FrameTransmitter, channel_notifier: mpsc::Receiver<RawFrame>) -> Self {
    Self {
      id,
      connection_notifier: Mutex::new(connection_notifier),
      channel_notifier: Some(channel_notifier),
      active: true,
      consumers: Arc::new(Mutex::new(Default::default())),
      sync_waiter_queue: Arc::new(Mutex::new(vec![]))
    }
  }

  pub async fn open(&mut self) -> Result<OpenOk> {
    let sync_waiter_queue = self.sync_waiter_queue.clone();
    let mut inner_rx = self.channel_notifier.take().unwrap();
    let consumers = self.consumers.clone();

    info!("[Channel] start incoming listener");
    tokio::spawn(async move {
      use crate::protocol::{basic};

      loop {
        if let Some(frame) = inner_rx.recv().await {
          match (frame.cid, frame.mid) {
            (60, basic::constants::METHOD_DELIVER) => {
              let payload: Deliver = frame.args.clone().try_into().unwrap();
              let consumers = consumers.lock().unwrap();
              let handler = consumers.get(&payload.consumer_tag).unwrap();
              handler.send(frame).unwrap();
            },
            _ => {
              //  todo: useless
              println!("***");
              println!("Unreachable code");
              println!("***");
              let mut sync_waiter_queue = sync_waiter_queue.lock().unwrap();
              sync_waiter_queue.pop().unwrap().send(frame).unwrap();
            }
          }
        } else {
          // todo: review action
          panic!("Channel receiver channel");
        }
      }
    });

    let method = Open::default();
    let response = self.invoke_sync_method(method).await?;
    Ok(response.args.try_into()?)
  }

  pub async fn declare_exchange(
    &self,
    name: String,
    ty: ExchangeType,
    durable: bool,
    passive: bool,
    auto_delete: bool,
    internal: bool,
    props: Option<Table>
  ) -> Result<()>
  {
    self.declare_exchange_with_builder(|builder| {
      builder.name(name);
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
    use crate::exchange::methods::{Declare};

    debug!("Declare exchange");
    let mut builder = ExchangeDeclareOptsBuilder::new();
    configure(&mut builder);
    self.invoke_sync_method(Declare::from(builder.build())).await?;

    Ok(())
  }

  pub async fn declare_queue(
    &self,
    name: &str,
    durable: bool,
    passive: bool,
    auto_delete: bool,
    exclusive: bool,
    props: Option<Table>
  ) -> Result<String> {
    self.declare_queue_with_builder(move |builder| {
      builder.name(name.to_string());
      builder.durable(durable);
      builder.passive(passive);
      builder.auto_delete(auto_delete);
      builder.exclusive(exclusive);
      builder.no_wait(false);
      builder.props(props.unwrap_or_else(|| Table::new()));
    }).await
  }

  pub async fn declare_queue_with_builder<F>(&self, configure: F) -> Result<String>
    where F: FnOnce(&mut QueueDeclareOptsBuilder) -> ()
  {
    use crate::queue::methods::{Declare, DeclareOk};

    debug!("Declare queue");
    let mut opts = QueueDeclareOptsBuilder::new();

    configure(&mut opts);

    let response = self.invoke_sync_method(Declare::from(opts.build())).await?;
    let payload: DeclareOk = response.args.try_into()?;

    Ok(payload.name)
  }

  pub async fn bind(&self, queue_name: String, exchange_name: String, routing_key: String) -> Result<()> {
    use crate::queue::methods::Bind;

    debug!("Bind queue: {} to: exchange {} with key: {}", queue_name.clone(), exchange_name.clone(), routing_key.clone());
    let payload = Bind {
      reserved1: 0,
      queue_name,
      exchange_name,
      routing_key,
      no_wait: 0,
      table: HashMap::new()
    };
    self.invoke_sync_method(payload).await?;

    Ok(())
  }

  pub async fn unbind(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
    use crate::queue::methods::Unbind;

    let payload = Unbind {
      reserved1: 0,
      queue_name: queue.to_string(),
      exchange_name: exchange.to_string(),
      routing_key: routing_key.to_string(),
      table: HashMap::new()
    };

    self.invoke_sync_method(payload).await?;

    Ok(())
  }

  pub async fn flow(&self, active: bool) -> Result<()> {
    use self::methods::Flow;

    debug!("Invoking channel {} Flow", self.id);
    if self.active == active {
      return Ok(())
    }

    self.invoke_sync_method(Flow { active: active as u8 }).await?;
    // self.active = active;

    Ok(())
  }

  pub async fn consume(&self, queue: String) -> Result<UnboundedReceiver<RawFrame>> {
    use crate::protocol::basic::methods::{Consume,ConsumeOk};

    info!("Consuming queue: {}", queue.clone());
    let payload = Consume {
      reserved1: 0,
      queue,
      tag: String::from(""),
      flags: 0,
      table: HashMap::new()
    };

    let response = self.invoke_sync_method(payload).await?;
    let payload: ConsumeOk = response.args.try_into()?;

    info!("Consume ok with tag: {}", payload.tag.clone());
    let (tx, rx) = mpsc::unbounded_channel();
    self.consumers.lock().unwrap().insert(payload.tag, tx);

    Ok(rx)
  }

  pub async fn publish(&self, exchange: String, routing_key: String, payload: Vec<u8>, fields: Option<Fields>) -> Result<()> {
    use crate::protocol::basic::methods::{Publish};

    info!("Publishing message");
    let method = Publish {
      reserved1: 0,
      exchange,
      routing_key,
      flags: 0,
    };
    let frame = Frame2 {
      ch: self.id,
      args: method,
      prop_fields: fields,
      body: Some(payload)
    };
    info!("Wait for the response");

    {
      let writer = self.connection_notifier.lock().unwrap();
      writer.send(frame.into()).await?;
    }

    info!("Message was published");

    Ok(())
  }

  pub async fn close(&self) -> Result<()> {
    use crate::channel::methods::Close;

    debug!("Closing channel {}", self.id);
    self.invoke_sync_method(Close {
      reply_code: 200,
      reply_text: "Closed".to_string(),
      class_id: 0,
      method_id: 0,
    }).await?;

    Ok(())
  }

  async fn invoke_sync_method<T: AmqpMethodArgs>(&self, args: T) -> Result<RawFrame> {
    let (tx, rx) = oneshot::channel::<RawFrame>();
    let sync_waiter_queue = self.sync_waiter_queue.clone();
    sync_waiter_queue.lock().unwrap().push(tx);

    let request: Frame2<T> = Frame2::new(self.id, args);

    let writer = self.connection_notifier.lock().unwrap();
    writer.send(request.into()).await?;

    Ok(rx.await?)
  }

  // async fn send<T: AmqpMethodArgs>(&self, args: T) -> Result<()> {
  //   let writer = self.connection_notifier.lock().unwrap();
  //
  //   let request: Frame2<T> = Frame2::new(self.id, args);
  //
  //   writer.send(request.into()).await?;
  //   Ok(())
  // }
  //
  // async fn send_with_body<T: AmqpMethodArgs>(&self, args: T, fields: Option<Fields>, body: Vec<u8>) -> Result<()> {
  //   let writer = self.connection_notifier.lock().unwrap();
  //
  //   let request: Frame2<T> = Frame2::new(self.id, args);
  //
  //   writer.send(request.into()).await?;
  //   Ok(())
  // }
  // async fn send_async(&self, request: RawFrame) -> Result<()> {
  //   let mut writer = self.writer.lock().unwrap();
  //   writer.send( request).await?;
  //
  //   Ok(())
  // }

  // todo: refactor result to avoid response prefix
  // pub fn handle_frame(&self, frame: MethodFrame) -> Result<()> {
  //   match frame.class_id {
  //     20 => {
  //       // self.handle_chan_frame(frame)?;
  //     },
  //     40 => {
  //       // self.handle_exchange_frame(frame)?;
  //     },
  //     50 => {
  //       // self.handle_queue_frame(frame)?;
  //     },
  //     60 => {
  //       // self.handle_basic_frame(frame)?;
  //     },
  //     _ => {
  //       panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
  //     }
  //   }
  //   Ok(())
  // }
  //
  // fn handle_chan_frame(&self, frame: MethodFrame) -> Result<()> {
  //   use crate::protocol::channel::{methods::{OpenOk, CloseOk}, constants::{METHOD_OPEN_OK, METHOD_CLOSE_OK}};
  //
  //   match frame.method_id {
  //     METHOD_OPEN_OK|METHOD_CLOSE_OK => {
  //       self.waiter_sender.lock().unwrap().send(frame)?;
  //     },
  //     _ => {
  //       panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
  //     }
  //   }
  //   Ok(())
  // }
  //
  // fn handle_exchange_frame(&self, frame: MethodFrame) -> Result<()> {
  //   use crate::protocol::exchange::{methods::{DeclareOk}, constants::{METHOD_DECLARE_OK}};
  //
  //   match frame.method_id {
  //     METHOD_DECLARE_OK => {
  //       self.waiter_sender.lock().unwrap().send(frame)?;
  //     },
  //     _ => {
  //       panic!("Received unknown method");
  //     }
  //   }
  //   Ok(())
  // }
  //
  // fn handle_queue_frame(&self, frame: MethodFrame) -> Result<()> {
  //   use crate::protocol::queue::{methods::{DeclareOk, BindOk, UnbindOk}, constants::{METHOD_DECLARE_OK, METHOD_BIND_OK, METHOD_UNBIND_OK}};
  //
  //   match frame.method_id {
  //     METHOD_DECLARE_OK|METHOD_BIND_OK|METHOD_UNBIND_OK => {
  //       self.waiter_sender.lock().unwrap().send(frame)?;
  //     },
  //     _ => {
  //       panic!("Received unknown queue method");
  //     }
  //   }
  //   Ok(())
  // }
  //
  // fn handle_basic_frame(&self, frame: MethodFrame) -> Result<()> {
  //   use crate::protocol::basic::{methods::{ConsumeOk,Deliver}, constants::{METHOD_CONSUME_OK, METHOD_DELIVER}};
  //
  //   match frame.method_id {
  //     METHOD_CONSUME_OK => {
  //       self.waiter_sender.lock().unwrap().send(frame)?;
  //     },
  //     METHOD_DELIVER => {
  //       let payload: Deliver = frame.body.clone().try_into()?;
  //       let consumers = self.consumers.lock().unwrap();
  //       let handler = consumers.get(&payload.consumer_tag).unwrap();
  //       handler.send(frame)?;
  //     },
  //     _ => {
  //       panic!("Received unknown queue method");
  //     }
  //   }
  //   Ok(())
  // }
  //
  // fn wait_for_response(&self) -> Result<MethodFrame> {
  //   Ok(self.waiter_channel.lock().unwrap().recv()?)
  // }
}
