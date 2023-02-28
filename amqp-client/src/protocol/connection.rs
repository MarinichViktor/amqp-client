use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::bail;
use log::{debug, info};
use url::Url;
use amqp_protocol::types::Property;
use crate::protocol::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::frame::{AmqFrame, AmqMethodFrame};
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
  pending: Arc<Mutex<HashMap<i16, AmqMethodFrame>>>
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
  pub fn from_uri(uri: &str) -> Result<Self> {
    let url = Url::parse(uri).unwrap();
    let host = if url.has_host() {
      url.host().unwrap().to_string()
    } else {
      String::from("localhost")
    };
    let port = url.port().unwrap_or_else(|| 5672);
    let (username, password) = if url.has_authority() {
      (
        url.username().to_string(),
        url.password().unwrap().to_string()
      )
    } else {
      bail!("Provide username and password in the connection url");
    };

    Ok(
      AmqConnection::new(
        host,
        port,
        username,
        password,
        url.path()[1..].to_string(),
      )
    )
  }

  pub fn new(host: String, port: u16, login: String, password: String, vhost: String) -> Self {
    let connection_url = format!("{}:{}", host.clone(), port);
    let options = ConnectionOpts { host, port, login, password, vhost };

    AmqConnection {
      amqp_stream: Arc::new(AmqpStream::new(connection_url)),
      channels: Arc::new(Mutex::new(HashMap::new())),
      id_allocator: IdAllocator::new(),
      options,
      pending: Arc::new(Mutex::new(HashMap::new()))
    }
  }

  pub fn connect(&mut self) -> Result<()> {
    use crate::protocol::connection::constants::PROTOCOL_HEADER;
    use crate::protocol::connection::methods as conn_methods;

    debug!("Connecting to the server");
    let mut reader = self.amqp_stream.reader.lock().unwrap();
    let mut writer = self.amqp_stream.writer.lock().unwrap();

    debug!("Sending ProtocolHeader");
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
    debug!("Sending StartOk");
    writer.invoke(0, start_ok_method)?;

    let frame = reader.next_method_frame()?;
    let tune_method: conn_methods::Tune = frame.body.try_into()?;

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.chan_max,
      frame_max: tune_method.frame_max,
      heartbeat: tune_method.heartbeat
    };
    debug!("Sending TuneOk");
    writer.invoke(0, tune_ok_method)?;

    let open_method = conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    };
    debug!("Sending OpenMethod");
    writer.invoke(0, open_method)?;
    let frame = reader.next_method_frame()?;
    let _open_ok_method: conn_methods::OpenOk = frame.body.try_into()?;
    debug!("Connected to the server");

    drop(reader);
    drop(writer);

    self.start_listener();
    Ok(())
  }

  pub fn create_channel(&mut self) -> Result<Arc<Channel>> {
    let id = self.id_allocator.allocate();
    debug!("Creating channel with id: {}", id);
    let chan = Channel::new(id, self.amqp_stream.clone());
    let chan = Arc::new(chan);
    self.channels.lock().unwrap().insert(chan.id, chan.clone());
    chan.open()?;

    Ok(chan)
  }

  pub fn start_listener(&mut self) {
    let reader = self.amqp_stream.reader.clone();
    let channels = self.channels.clone();
    let pending = self.pending.clone();

    std::thread::spawn(move || {
      let mut reader = reader.lock().unwrap();

      while let Ok(frame) = reader.next_frame() {
        debug!("Received AmqFrame");
        match frame {
          AmqFrame::Method(method_frame) => {
            // todo: check if method expects some body
            debug!("Received method frame: channel {}, class_id {}, method_id: {}", method_frame.chan, method_frame.class_id, method_frame.method_id);
            if method_frame.class_id == 60 && method_frame.method_id == 60 {
              pending.lock().unwrap().insert(method_frame.chan, method_frame);
            } else {
              channels.lock().unwrap()[&method_frame.chan].handle_frame(method_frame).unwrap();
            }
          },
          AmqFrame::Header(header) => {
            let chan = header.chan;
            debug!("Received header frame: channel {}, class_id {}", header.chan, header.class_id);
            pending.lock().unwrap().get_mut(&chan).unwrap().content_header = Some(header);
          },
          AmqFrame::Body(mut frame) => {
            debug!("Received body frame");
            let mut pend_map = pending.lock().unwrap();

            let mut frame_ref = pend_map.remove(&frame.chan).unwrap();

            let mut content_body = frame_ref.content_body.take().unwrap_or_else(|| vec![]);
            content_body.append(&mut frame.body);

            let curr_len = content_body.len();
            let body_len = match &frame_ref.content_header {
              Some(x) => x.body_len,
              _ => panic!("failed to get content header")
            };
            frame_ref.content_body = Some(content_body);

            if curr_len as i64 == body_len {
              debug!("Received all body, executing");
              channels.lock().unwrap()[&frame.chan].handle_frame(frame_ref).unwrap();
            } else {
              debug!("Waiting for next body frame... curr{}, expected{}", curr_len, body_len);
              pend_map.insert(frame.chan, frame_ref);
            }
          }
          _ => {
            // panic!("Unsupported frame type")
          }
        }
      }
    });
  }
}
