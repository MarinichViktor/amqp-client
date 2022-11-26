use std::collections::HashMap;
use std::fs::read;
use std::sync::{Arc, Mutex};
use anyhow::bail;
use log::info;
use amqp_protocol::types::Property;
use crate::protocol::frame::{Method, MethodFrame};
use crate::protocol::methods::connection::{Open, Start, StartOk, TuneOk};
use crate::protocol::stream::ConnectionOpts;
use super::protocol::stream::{AmqpStream};
use crate::response;

static PROTOCOL_HEADER: [u8;8] = [65,77,81,80,0,0,9,1];

pub struct DefaultChan {
}

pub struct Connection {
  amqp_stream: Arc<AmqpStream>
}

impl Connection {
  pub fn new(host: String, port: u16, login: String, password: String) -> Self {
    let stream = AmqpStream::new(
      //  todo: move connection opts on Connection level
      ConnectionOpts {
        host,
        port,
        login,
        password
      }
    );

    Connection {
      amqp_stream: Arc::new(stream)
    }
  }

  pub fn connect(&mut self) -> response::Result<()> {
    let mut reader = self.amqp_stream.reader.lock().unwrap();
    let mut writer = self.amqp_stream.writer.lock().unwrap();
    writer.send_raw(&PROTOCOL_HEADER)?;

    let MethodFrame { chan: _, payload: method } = reader.next_method_frame()?;
    // todo: write some helper for a such common case
    let start_method = match method {
      Method::ConnStart(start) => {
        start
      }
      _ => {
        bail!("Expected ConnStart method");
      }
    };

    // todo: provide some meaningful values
    let mut client_properties = HashMap::new();
    client_properties.insert("product".to_string(), Property::LongStr("simpleapp123123asd ".to_string()));
    client_properties.insert("platform".to_string(), Property::LongStr("Erlang/OTP 24.3.4".to_string()));
    client_properties.insert("copyright".to_string(), Property::LongStr("param-pam-pamqweqwe".to_string()));
    client_properties.insert("information".to_string(), Property::LongStr("Licensed under the MPL 2.0".to_string()));

    // todo: use values from conn options
    let start_ok_method = StartOk {
      properties: client_properties,
      mechanism: "PLAIN".to_string(),
      response: "\x00user\x00password".to_string(),
      locale: "en_US".to_string()
    };
    writer.invoke(0, start_ok_method)?;

    let MethodFrame { payload: method, .. } = reader.next_method_frame()?;
    let tune_method = match method {
      Method::ConnTune(tune) => {
        tune
      }
      _ => {
        bail!("Expected ConnTune method");
      }
    };

    // todo: use values from conn options
    let tune_ok_method = TuneOk {
      chan_max: tune_method.chan_max,
      frame_max: tune_method.frame_max,
      heartbeat: tune_method.heartbeat
    };
    writer.invoke(0, tune_ok_method)?;

    let open_method = Open {
      vhost: "my_vhost".to_string(),
      ..Open::default()
    };
    writer.invoke(0, open_method)?;

    Ok(())
  }
}
