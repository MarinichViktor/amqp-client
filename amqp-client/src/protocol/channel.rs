use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::stream::AmqpStream;
use crate::{Result};
use std::sync::mpsc::{channel, Receiver, Sender};
use log::info;
use crate::protocol::exchange::{ExchangeDeclareOpts, ExchangeDeclareOptsBuilder};
use crate::protocol::frame::{AmqMethodFrame};
use crate::protocol::queue::QueueDeclareOptsBuilder;

pub mod methods;
pub mod constants;

// todo: to be used
pub struct AmqChannel {
  pub id: i16,
  amqp_stream: Arc<AmqpStream>,
  waiter_channel: Mutex<Receiver<()>>,
  waiter_sender: Mutex<Sender<()>>,
  active: bool,
}

impl AmqChannel {
  pub(crate) fn new(id: i16, amqp_stream: Arc<AmqpStream>) -> Self {
    let (sender, receiver) = channel();
    Self {
      id,
      amqp_stream,
      waiter_channel: Mutex::new(receiver),
      waiter_sender: Mutex::new(sender),
      active: true
    }
  }

  // todo: refactor result to avoid response prefix
  pub fn handle_frame(&self, frame: AmqMethodFrame) -> Result<()> {
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

  fn handle_chan_frame(&self, frame: AmqMethodFrame) -> Result<()> {
    use crate::protocol::channel::{methods::{OpenOk, CloseOk}, constants::{METHOD_OPEN_OK, METHOD_CLOSE_OK}};

    match frame.method_id {
      METHOD_OPEN_OK => {
        let payload: OpenOk = frame.body.try_into()?;
        info!("Received open ok method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      METHOD_CLOSE_OK => {
        let payload: CloseOk = frame.body.try_into()?;
        info!("Received close ok method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      _ => {
        panic!("Received unknown method {}, {}", frame.class_id, frame.method_id);
      }
    }
    Ok(())
  }

  fn handle_exchange_frame(&self, frame: AmqMethodFrame) -> Result<()> {
    use crate::protocol::exchange::{methods::{DeclareOk}, constants::{METHOD_DECLARE_OK}};

    match frame.method_id {
      METHOD_DECLARE_OK => {
        let payload: DeclareOk = frame.body.try_into()?;
        info!("Received declare ok method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      _ => {
        panic!("Received unknown method");
      }
    }
    Ok(())
  }

  fn handle_queue_frame(&self, frame: AmqMethodFrame) -> Result<()> {
    use crate::protocol::queue::{methods::{DeclareOk, BindOk, UnbindOk}, constants::{METHOD_DECLARE_OK, METHOD_BIND_OK, METHOD_UNBIND_OK}};

    match frame.method_id {
      METHOD_DECLARE_OK => {
        let payload: DeclareOk = frame.body.try_into()?;
        info!("Received Queue#declareOk method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      METHOD_BIND_OK => {
        let payload: BindOk = frame.body.try_into()?;
        info!("Received Queue#bindOk method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      METHOD_UNBIND_OK => {
        let payload: UnbindOk = frame.body.try_into()?;
        info!("Received Queue#unbindOk method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      _ => {
        panic!("Received unknown queue method");
      }
    }
    Ok(())
  }

  fn handle_basic_frame(&self, frame: AmqMethodFrame) -> Result<()> {
    use crate::protocol::basic::{methods::{ConsumeOk,Deliver}, constants::{METHOD_CONSUME_OK, METHOD_DELIVER}};

    match frame.method_id {
      METHOD_CONSUME_OK => {
        let payload: ConsumeOk = frame.body.try_into()?;
        info!("Received Basic#consumeOk method {:?}", payload.tag);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      METHOD_DELIVER => {
        let payload: Deliver = frame.body.try_into()?;
        info!("Received Basic#deliver method ***");
        let bd = frame.content_body.unwrap();
        println!("Body {:?}", String::from_utf8(bd));
        self.waiter_sender.lock().unwrap().send(())?;
      },
      _ => {
        panic!("Received unknown queue method");
      }
    }
    Ok(())
  }

  pub fn open(&self) -> Result<()> {
    use crate::protocol::channel::methods::Open;

    info!("Invoking Open");
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Open::default())?;
    self.wait_for_response()?;

    Ok(())
  }

  pub fn flow(&mut self, active: bool) -> Result<()> {
    use crate::protocol::channel::methods::Flow;

    info!("Invoking Flow");
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

    info!("Invoking close method");
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

  pub fn declare_exchange<F>(&self, configure: F) -> Result<String>
    where F: FnOnce(&mut ExchangeDeclareOptsBuilder) -> ()
  {
    let mut builder = ExchangeDeclareOptsBuilder::new();
    configure(&mut builder);
    self.declare_exchange_with_opts(builder.build())
  }

  pub fn declare_exchange_with_opts(&self, opts: ExchangeDeclareOpts) -> Result<String> {
    use crate::protocol::exchange::methods::Declare;
    let name = opts.name.clone();

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Declare::from(opts))?;
    self.wait_for_response()?;

    Ok(name)
  }

  pub fn declare_queue<F>(&self, configure: F) -> Result<String>
    where F: Fn(&mut QueueDeclareOptsBuilder) -> ()
  {
    use crate::protocol::queue::methods::Declare;

    let mut opts = QueueDeclareOptsBuilder::new();

    configure(&mut opts);
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Declare::from(opts.build()))?;

    self.wait_for_response()?;

    Ok(String::from("qwe"))
  }

  pub fn bind(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<()> {
    use crate::protocol::queue::methods::Bind;

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Bind {
      reserved1: 0,
      queue_name: queue.to_string(),
      exchange_name: exchange.to_string(),
      routing_key: routing_key.to_string(),
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

  pub fn consume(&self, queue: String, tag: String) -> Result<()> {
    use crate::protocol::basic::methods::Consume;

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Consume {
      reserved1: 0,
      queue,
      tag,
      flags: 0,
      table: HashMap::new()
    })?;
    self.wait_for_response()?;

    Ok(())
  }

  fn wait_for_response(&self) -> Result<()> {
    Ok(self.waiter_channel.lock().unwrap().recv()?)
  }
}
