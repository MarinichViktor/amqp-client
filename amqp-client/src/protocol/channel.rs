use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::stream::AmqpStream;
use crate::{Result};
use std::sync::mpsc::{channel, Receiver, Sender};
use log::{debug};
use amqp_protocol::types::{Table};
use crate::protocol::exchange::{ExchangeDeclareOptsBuilder, ExchangeType};
use crate::protocol::frame::{MethodFrame};
use crate::protocol::queue::{QueueDeclareOptsBuilder};

pub mod methods;
pub mod constants;

pub struct AmqChannel {
  pub id: i16,
  amqp_stream: Arc<AmqpStream>,
  waiter_channel: Mutex<Receiver<MethodFrame>>,
  waiter_sender: Mutex<Sender<MethodFrame>>,
  active: bool,
  consumers: Arc<Mutex<HashMap<String, Sender<MethodFrame>>>>
}

impl AmqChannel {
  pub(crate) fn new(id: i16, amqp_stream: Arc<AmqpStream>) -> Self {
    let (sender, receiver) = channel();
    Self {
      id,
      amqp_stream,
      waiter_channel: Mutex::new(receiver),
      waiter_sender: Mutex::new(sender),
      active: true,
      consumers: Arc::new(Mutex::new(HashMap::new()))
    }
  }

  pub fn open(&self) -> Result<()> {
    use crate::protocol::channel::methods::Open;

    debug!("Opening channel {}", self.id);
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Open::default())?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn flow(&mut self, active: bool) -> Result<()> {
    use crate::protocol::channel::methods::Flow;

    debug!("Invoking channel {} Flow", self.id);
    if self.active == active {
      return Ok(())
    }

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Flow {
      active: active as u8
    })?;
    self.wait_for_response()?;
    self.active = active;

    Ok(())
  }

  pub fn close(&self) -> Result<()> {
    use crate::protocol::channel::methods::Close;

    debug!("Closing channel {}", self.id);
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Close {
      reply_code: 200,
      reply_text: "Closed".to_string(),
      class_id: 0,
      method_id: 0,
    })?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn exchange_declare(
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
    })
  }

  pub fn declare_exchange_with_builder<F>(&self, configure: F) -> Result<()>
    where F: FnOnce(&mut ExchangeDeclareOptsBuilder) -> ()
  {
    debug!("Declare exchange");
    use crate::protocol::exchange::methods::{Declare};
    let mut builder = ExchangeDeclareOptsBuilder::new();
    configure(&mut builder);
    let opts = builder.build();

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Declare::from(opts))?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn queue_declare(
    &self,
    name: &str,
    durable: bool,
    passive: bool,
    auto_delete: bool,
    exclusive: bool,
    props: Option<Table>
  ) -> Result<String> {
    self.queue_declare_with_builder(move |builder| {
      builder.name(name.to_string());
      builder.durable(durable);
      builder.passive(passive);
      builder.auto_delete(auto_delete);
      builder.exclusive(exclusive);
      builder.no_wait(false);
      builder.props(props.unwrap_or_else(|| Table::new()));
    })
  }

  pub fn queue_declare_with_builder<F>(&self, configure: F) -> Result<String>
    where F: FnOnce(&mut QueueDeclareOptsBuilder) -> ()
  {
    use crate::protocol::queue::methods::{Declare, DeclareOk};

    debug!("Declare queue");
    let mut opts = QueueDeclareOptsBuilder::new();

    configure(&mut opts);
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Declare::from(opts.build()))?;

    let resp_frame = self.wait_for_response()?;
    let payload: DeclareOk = resp_frame.body.try_into()?;

    Ok(payload.name)
  }

  pub fn bind(&self, queue_name: String, exchange_name: String, routing_key: String) -> Result<()> {
    use crate::protocol::queue::methods::Bind;

    debug!("Bind queue: {} to: exchange {} with key: {}", queue_name.clone(), exchange_name.clone(), routing_key.clone());
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Bind {
      reserved1: 0,
      queue_name,
      exchange_name,
      routing_key,
      no_wait: 0,
      table: HashMap::new()
    })?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn unbind(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
    use crate::protocol::queue::methods::Unbind;

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Unbind {
      reserved1: 0,
      queue_name: queue.to_string(),
      exchange_name: exchange.to_string(),
      routing_key: routing_key.to_string(),
      table: HashMap::new()
    })?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn consume(&self, queue: String) -> Result<Receiver<MethodFrame>> {
    use crate::protocol::basic::methods::{Consume,ConsumeOk};

    debug!("Consuming queue: {}", queue.clone());
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Consume {
      reserved1: 0,
      queue,
      tag: String::from(""),
      flags: 0,
      table: HashMap::new()
    })?;
    let resp_frame = self.wait_for_response()?;
    let payload: ConsumeOk = resp_frame.body.try_into()?;
    debug!("Consume ok with tag: {}", payload.tag.clone());
    let (tx, rx) = channel();
    self.consumers.lock().unwrap().insert(payload.tag, tx);

    Ok(rx)
  }

  // todo: refactor result to avoid response prefix
  pub fn handle_frame(&self, frame: MethodFrame) -> Result<()> {
    match frame.class_id {
      20 => {
        self.handle_chan_frame(frame)?;
      },
      40 => {
        self.handle_exchange_frame(frame)?;
      },
      50 => {
        self.handle_queue_frame(frame)?;
      },
      60 => {
        self.handle_basic_frame(frame)?;
      },
      _ => {
        panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
      }
    }
    Ok(())
  }

  fn handle_chan_frame(&self, frame: MethodFrame) -> Result<()> {
    use crate::protocol::channel::{methods::{OpenOk, CloseOk}, constants::{METHOD_OPEN_OK, METHOD_CLOSE_OK}};

    match frame.method_id {
      METHOD_OPEN_OK|METHOD_CLOSE_OK => {
        self.waiter_sender.lock().unwrap().send(frame)?;
      },
      _ => {
        panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
      }
    }
    Ok(())
  }

  fn handle_exchange_frame(&self, frame: MethodFrame) -> Result<()> {
    use crate::protocol::exchange::{methods::{DeclareOk}, constants::{METHOD_DECLARE_OK}};

    match frame.method_id {
      METHOD_DECLARE_OK => {
        self.waiter_sender.lock().unwrap().send(frame)?;
      },
      _ => {
        panic!("Received unknown method");
      }
    }
    Ok(())
  }

  fn handle_queue_frame(&self, frame: MethodFrame) -> Result<()> {
    use crate::protocol::queue::{methods::{DeclareOk, BindOk, UnbindOk}, constants::{METHOD_DECLARE_OK, METHOD_BIND_OK, METHOD_UNBIND_OK}};

    match frame.method_id {
      METHOD_DECLARE_OK|METHOD_BIND_OK|METHOD_UNBIND_OK => {
        self.waiter_sender.lock().unwrap().send(frame)?;
      },
      _ => {
        panic!("Received unknown queue method");
      }
    }
    Ok(())
  }

  fn handle_basic_frame(&self, frame: MethodFrame) -> Result<()> {
    use crate::protocol::basic::{methods::{ConsumeOk,Deliver}, constants::{METHOD_CONSUME_OK, METHOD_DELIVER}};

    match frame.method_id {
      METHOD_CONSUME_OK => {
        self.waiter_sender.lock().unwrap().send(frame)?;
      },
      METHOD_DELIVER => {
        let payload: Deliver = frame.body.clone().try_into()?;
        let consumers = self.consumers.lock().unwrap();
        let handler = consumers.get(&payload.consumer_tag).unwrap();
        handler.send(frame)?;
      },
      _ => {
        panic!("Received unknown queue method");
      }
    }
    Ok(())
  }

  fn wait_for_response(&self) -> Result<MethodFrame> {
    Ok(self.waiter_channel.lock().unwrap().recv()?)
  }
}
