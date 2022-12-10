use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::stream::AmqpStream;
use crate::{Result};
use std::sync::mpsc::{channel, Receiver, Sender};
use log::info;
use amqp_protocol::types::Table;
use crate::protocol::exchange::{ExchangeDeclareOpts, ExchangeDeclareOptsBuilder, ExchangeType};
use crate::protocol::frame::{AmqMethodFrame};

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
      }
      _ => {
        panic!("Received unknown method");
      }
    }
    Ok(())
  }

  fn handle_chan_frame(&self, frame: AmqMethodFrame) -> Result<()> {
    use crate::protocol::channel::{methods::{OpenOk}, constants::{METHOD_OPEN_OK}};

    match frame.method_id {
      METHOD_OPEN_OK => {
        let payload: OpenOk = frame.body.try_into()?;
        info!("Received open ok method {:?}", payload);
        self.waiter_sender.lock().unwrap().send(())?;
      },
      _ => {
        panic!("Received unknown method");
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
    use crate::protocol::channel::methods::CloseOk;

    info!("Invoking close method");
    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, CloseOk::default())?;
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
    let name = opts.name;

    let mut stream_writer = self.amqp_stream.writer.lock().unwrap();
    stream_writer.invoke(self.id, Declare::new_with_config(
      name.clone(),
      opts.ty,
      opts.passive,
      opts.durable,
      opts.auto_delete,
      opts.internal,
      opts.no_wait,
      opts.props
    ))?;
    self.wait_for_response()?;

    Ok(name)
  }

  fn wait_for_response(&self) -> Result<()> {
    Ok(self.waiter_channel.lock().unwrap().recv()?)
  }
}
