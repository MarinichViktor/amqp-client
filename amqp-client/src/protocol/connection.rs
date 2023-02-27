use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{debug, info};
use amqp_protocol::types::Property;
use crate::protocol::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::frame::{AmqFrame};
use crate::protocol::stream::{AmqpStream};
use crate::{Result, Channel};
use crate::utils::IdAllocator;

pub mod constants;
pub mod methods;

pub struct AmqConnection {
  pub options: ConnectionOpts,
  amqp_stream: Arc<AmqpStream>,
  channels: Arc<Mutex<HashMap<i16,Arc<Channel>>>>,
  id_allocator: IdAllocator,
}

#[derive(Clone)]
pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String,
  pub vhost: String,
}

impl AmqConnection {
  pub fn new(host: String, port: u16, login: String, password: String, vhost: String) -> Self {
    let connection_url = format!("{}:{}", host.clone(), port);
    let options = ConnectionOpts { host, port, login, password, vhost };

    AmqConnection {
      amqp_stream: Arc::new(AmqpStream::new(connection_url)),
      channels: Arc::new(Mutex::new(HashMap::new())),
      id_allocator: IdAllocator::new(),
      options
    }
  }

  pub fn connect(&mut self) -> Result<()> {
    use crate::protocol::connection::constants::PROTOCOL_HEADER;
    use crate::protocol::connection::methods as conn_methods;

    let mut reader = self.amqp_stream.reader.lock().unwrap();
    let mut writer = self.amqp_stream.writer.lock().unwrap();

    info!("Connecting to the server");
    writer.send_raw(&PROTOCOL_HEADER)?;

    let frame = reader.next_method_frame()?;
    let _start_method: conn_methods::Start = frame.body.try_into()?;

    let client_properties = HashMap::from([
      ("product".to_string(), Property::LongStr(PRODUCT.to_string())),
      ("platform".to_string(), Property::LongStr(PLATFORM.to_string())),
      ("copyright".to_string(), Property::LongStr(COPYRIGHT.to_string())),
      ("information".to_string(), Property::LongStr(INFORMATION.to_string()))
    ]);
    let start_ok_method = conn_methods::StartOk {
      properties: client_properties,
      mechanism: DEFAULT_AUTH_MECHANISM.to_string(),
      response: format!("\x00{}\x00{}", self.options.login.as_str(), self.options.password),
      locale: DEFAULT_LOCALE.to_string()
    };
    writer.invoke(0, start_ok_method)?;

    let frame = reader.next_method_frame()?;
    let tune_method: conn_methods::Tune = frame.body.try_into()?;

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.chan_max,
      frame_max: tune_method.frame_max,
      heartbeat: tune_method.heartbeat
    };
    writer.invoke(0, tune_ok_method)?;

    let open_method = conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    };
    writer.invoke(0, open_method)?;
    let frame = reader.next_method_frame()?;
    let _open_ok_method: conn_methods::OpenOk = frame.body.try_into()?;
    info!("Connected to the server");

    drop(reader);
    drop(writer);

    self.start_listener();
    Ok(())
  }

  pub fn create_channel(&mut self) -> Result<Arc<Channel>> {
    let id = self.id_allocator.allocate();
    let chan = Channel::new(id, self.amqp_stream.clone());
    let chan = Arc::new(chan);
    self.channels.lock().unwrap().insert(chan.id, chan.clone());
    chan.open()?;

    Ok(chan)
  }

  pub fn start_listener(&self) {
    let reader = self.amqp_stream.reader.clone();
    let channels = self.channels.clone();

    std::thread::spawn(move || {
      let mut reader = reader.lock().unwrap();

      while let Ok(frame) = reader.next_frame() {
        debug!("Received AmqFrame");
        match frame {
          AmqFrame::Method(method_frame) => {
            // todo: check if method expects some body
            debug!("Received method frame: channel {}, class_id {}, method_id: {}", method_frame.chan, method_frame.class_id, method_frame.method_id);
            channels.lock().unwrap()[&method_frame.chan].handle_frame(method_frame).unwrap();
          },
          // _ => {
          //   panic!("Unsupported frame type")
          // }
        }
      }
    });
  }
}
