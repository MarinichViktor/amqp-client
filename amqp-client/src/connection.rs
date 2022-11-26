use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::frame::{Frame, MethodFrame};
use crate::protocol::stream::ConnectionOpts;
use super::protocol::stream::{AmqpStream};
use crate::response;

pub struct DefaultChan {

}

pub struct Connection {
  amqp_stream: Arc<Mutex<AmqpStream>>
}

impl Connection {
  pub fn new(host: String, port: u16, login: String, password: String) -> Self {
    let stream = AmqpStream::new(
      ConnectionOpts {
        host,
        port,
        login,
        password
      }
    );

    Connection {
      amqp_stream: Arc::new(Mutex::new(stream))
    }
  }

  pub fn connect(&mut self) -> response::Result<()> {
    let mut amqp_stream = self.amqp_stream.lock().unwrap();
    amqp_stream.protocol_header()?;
    let frame = amqp_stream.next_method_frame()?;

    // let MethodFrame::Connection(
    //   Start {
    //     ver_major,
    //     ver_minor,
    //     properties,
    //     mechanisms,
    //     locales,
    //     ..
    //   }
    // ) = amqp_stream.next_method_frame()?;
    //
    // self.send_start_ok()?;
    Ok(())
  }


  fn send_start_ok(&mut self) -> response::Result<()> {
    let mut amqp_stream = self.amqp_stream.lock().unwrap();

    let mut client_properties = HashMap::new();
    client_properties.insert("product".to_string(), ServerProperty::ShortStr("simpleapp123123asd asd asdasd simpleapp123123asd asd asdasd".to_string()));
    client_properties.insert("platform".to_string(), ServerProperty::ShortStr("Erlang/OTP 24.3.4".to_string()));
    client_properties.insert("copyright".to_string(), ServerProperty::ShortStr("param-pam-pamqweqwe".to_string()));
    client_properties.insert("information".to_string(), ServerProperty::ShortStr("Licensed under the MPL 2.0".to_string()));

    amqp_stream.start_ok(client_properties, "PLAIN".to_string(), "\x00user\x00password".to_string(), "en_US".to_string())?;

    Ok(())
  }
}
