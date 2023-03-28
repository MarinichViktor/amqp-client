use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::{ExchangeType, Result};
use log::{debug, info};
use crate::protocol::connection::{FrameSender};
use crate::protocol::frame::{MethodFrame};
use tokio::sync::{oneshot, mpsc};
use amqp_protocol::types::Table;
use crate::protocol::channel::methods::{Open, OpenOk};
use crate::protocol::exchange::ExchangeDeclareOptsBuilder;
use crate::protocol::queue::QueueDeclareOptsBuilder;

pub mod methods;
pub mod constants;

pub type  ChannelSender = mpsc::Sender<(i32, Vec<u8>)>;

pub struct AmqChannel {
  pub id: i16,
  writer: FrameSender,
  // waiter_channel: Mutex<Receiver<MethodFrame>>,
  // waiter_sender: Mutex<Sender<MethodFrame>>,
  pub inner_tx: mpsc::Sender<MethodFrame>,
  pub inner_rx: Option<mpsc::Receiver<MethodFrame>>,
  active: bool,
  // consumers: Arc<Mutex<HashMap<String, Sender<MethodFrame>>>>,
  sync_waiter_queue: Arc<Mutex<Vec<oneshot::Sender<MethodFrame>>>>
}

impl AmqChannel {
  pub(crate) fn new(id: i16, writer: FrameSender) -> Self {
    let (inner_tx, inner_rx) = mpsc::channel(64);

    Self {
      id,
      writer,
      inner_tx,
      inner_rx: Some(inner_rx),
      active: true,
      sync_waiter_queue: Arc::new(Mutex::new(vec![]))
      // waiter_channel: Mutex::new(receiver),
      // waiter_sender: Mutex::new(sender),
      // consumers: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn open(&mut self) -> Result<OpenOk> {
    let sync_waiter_queue = self.sync_waiter_queue.clone();
    let mut inner_rx = self.inner_rx.take().unwrap();

    info!("[Channel] start incoming listener");
    tokio::spawn(async move {
      loop {
        if let Some(frame) = inner_rx.recv().await {
          let mut sync_waiter_queue = sync_waiter_queue.lock().unwrap();
          sync_waiter_queue.pop().unwrap().send(frame).unwrap();
        } else {
          // todo: review action
          println!("Channel receiver exited");
          panic!("Exited channel");
        }
      }
    });

    let response = self.send_sync(Open::default()).await?;
    Ok(response.body.try_into()?)
  }

  pub async fn declare_exchange(
    &mut self,
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

  pub async fn declare_exchange_with_builder<F>(&mut self, configure: F) -> Result<()>
    where F: FnOnce(&mut ExchangeDeclareOptsBuilder) -> ()
  {
    debug!("Declare exchange");
    use crate::protocol::exchange::methods::{Declare};

    let mut builder = ExchangeDeclareOptsBuilder::new();
    configure(&mut builder);
    let opts = builder.build();
    self.send_sync(Declare::from(opts)).await?;

    Ok(())
  }

  pub async fn declare_queue(
    &mut self,
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

  pub async fn declare_queue_with_builder<F>(&mut self, configure: F) -> Result<String>
    where F: FnOnce(&mut QueueDeclareOptsBuilder) -> ()
  {
    use crate::protocol::queue::methods::{Declare, DeclareOk};

    debug!("Declare queue");
    let mut opts = QueueDeclareOptsBuilder::new();

    configure(&mut opts);

    let response = self.send_sync(Declare::from(opts.build())).await?;
    let payload: DeclareOk = response.body.try_into()?;

    Ok(payload.name)
  }

  pub async fn bind(&mut self, queue_name: String, exchange_name: String, routing_key: String) -> Result<()> {
    use crate::protocol::queue::methods::Bind;

    debug!("Bind queue: {} to: exchange {} with key: {}", queue_name.clone(), exchange_name.clone(), routing_key.clone());
    let payload = Bind {
      reserved1: 0,
      queue_name,
      exchange_name,
      routing_key,
      no_wait: 0,
      table: HashMap::new()
    };
    self.send_sync(payload).await?;

    Ok(())
  }

  pub async fn unbind(&mut self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
    use crate::protocol::queue::methods::Unbind;

    let payload = Unbind {
      reserved1: 0,
      queue_name: queue.to_string(),
      exchange_name: exchange.to_string(),
      routing_key: routing_key.to_string(),
      table: HashMap::new()
    };

    self.send_sync(payload).await?;

    Ok(())
  }

  pub async fn flow(&mut self, active: bool) -> Result<()> {
    use crate::protocol::channel::methods::Flow;

    debug!("Invoking channel {} Flow", self.id);
    if self.active == active {
      return Ok(())
    }

    self.send_sync(Flow { active: active as u8 }).await?;
    self.active = active;

    Ok(())
  }

  pub async fn close(&mut self) -> Result<()> {
    use crate::protocol::channel::methods::Close;

    debug!("Closing channel {}", self.id);
    self.send_sync(Close {
      reply_code: 200,
      reply_text: "Closed".to_string(),
      class_id: 0,
      method_id: 0,
    }).await?;

    Ok(())
  }

  async fn send_sync<T>(&mut self, request: T) -> Result<MethodFrame>
    where T: TryInto<Vec<u8>, Error = crate::Error>
  {
    let (tx, rx) = oneshot::channel::<MethodFrame>();
    let sync_waiter_queue = self.sync_waiter_queue.clone();
    sync_waiter_queue.lock().unwrap().push(tx);

    self.writer.send(self.id, request).await?;

    Ok(rx.await?)
  }

  //


  // pub fn consume(&self, queue: String) -> Result<Receiver<MethodFrame>> {
  //   use crate::protocol::basic::methods::{Consume,ConsumeOk};
  //
  //   debug!("Consuming queue: {}", queue.clone());
  //   let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
  //   stream_writer.invoke(self.id, Consume {
  //     reserved1: 0,
  //     queue,
  //     tag: String::from(""),
  //     flags: 0,
  //     table: HashMap::new()
  //   })?;
  //   let resp_frame = self.wait_for_response()?;
  //   let payload: ConsumeOk = resp_frame.body.try_into()?;
  //   debug!("Consume ok with tag: {}", payload.tag.clone());
  //   let (tx, rx) = channel();
  //   self.consumers.lock().unwrap().insert(payload.tag, tx);
  //
  //   Ok(rx)
  // }
  //
  // todo: refactor result to avoid response prefix
  pub fn handle_frame(&self, frame: MethodFrame) -> Result<()> {
    match frame.class_id {
      20 => {
        // self.handle_chan_frame(frame)?;
      },
      40 => {
        // self.handle_exchange_frame(frame)?;
      },
      50 => {
        // self.handle_queue_frame(frame)?;
      },
      60 => {
        // self.handle_basic_frame(frame)?;
      },
      _ => {
        panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
      }
    }
    Ok(())
  }
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
