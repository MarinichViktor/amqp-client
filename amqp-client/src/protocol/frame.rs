use std::io::{Cursor, Write};
use crate::protocol::decoder::Decode;
use crate::protocol::encoder::Encode;
use crate::protocol::method::{PropTable, ServerProperty};
use crate::response;

pub mod encoding;
pub mod decoding;

#[derive(Debug)]
pub enum Frame {
  Method(MethodFrame),
  Heartbeat
}

#[derive(Debug)]
pub enum MethodFrame {
  Connection(ConnectionMethod)
}

// #[derive(Debug)]
// #[amqp::method_frame]
// pub struct StartMethod {
//   #[short]
//   channel: i16,
//   #[byte]
//   ver_major: u8,
//   #[byte]
//   ver_minor: u8,
//   #[prop_table]
//   properties: PropTable,
//   #[long_str]
//   mechanisms: String,
//   #[long_str]
//   locales: String,
// }

#[derive(Debug)]
pub enum ConnectionMethod {
  Start {
    channel: i16,
    ver_major: u8,
    ver_minor: u8,
    properties: PropTable,
    mechanisms: Vec<String>,
    locales: Vec<String>,
  },
  StartOk {
    channel: i16,
    properties: PropTable,
    mechanism: String,
    response: String,
    locale: String,
  },
}

impl MethodFrame {
  // pub fn to_bytes(self) -> response::Result<Vec<u8>> {
  //   let mut frame_bytes = vec![];
  //   // method_type
  //   frame_bytes.write_byte(1)?;
  //   // method_type
  //
  //   match self {
  //     MethodFrame::Connection(conn_method) => {
  //       match conn_method {
  //         ConnectionMethod::StartOk { channel, properties, mechanism,
  //           response, locale } => {
  //           frame_bytes.write_short(channel)?;
  //
  //           let mut args_buff = vec![];
  //           args_buff.write_short(10)?;
  //           args_buff.write_short(11)?;
  //
  //           args_buff.write_prop_table(properties)?;
  //           args_buff.write_short_str(mechanism)?;
  //           args_buff.write_long_str(response)?;
  //           args_buff.write_short_str(locale)?;
  //
  //           Encode::write_int(&mut frame_bytes, args_buff.len() as i32)?;
  //           frame_bytes.write(& args_buff)?;
  //           frame_bytes.write_byte(206)?;
  //         }
  //         _ => {
  //           panic!("Not implemented")
  //         }
  //       }
  //     }
  //   };
  //
  //   Ok(frame_bytes)
  // }
}

// todo: to be removed
#[derive(Debug)]
pub struct MethodFrames {
  pub channel: i16,
  pub class_id: i16,
  pub method_id: i16,
  pub body: Vec<u8>
}

// impl MethodFrames {
//   pub fn method_args(&self) -> response::Result<Vec<ServerProperty>> {
//     let mut args_buff = Cursor::new(self.body.clone());
//     let mut args = vec![];
//
//     match self.class_id {
//       10 => {
//         match self.method_id {
//           10 => {
//             // ver_major
//             args.push(ServerProperty::Byte(args_buff.read_byte()?));
//             // ver_minor
//             args.push(ServerProperty::Byte(args_buff.read_byte()?));
//             // server_properties
//             args.push(ServerProperty::PropTable(args_buff.read_prop_table()?));
//             // mechanisms
//             args.push(ServerProperty::LongStr(args_buff.read_long_str()?));
//             // locales
//             args.push(ServerProperty::LongStr(args_buff.read_long_str()?));
//           }
//           _ => {
//             panic!("Unknown method_id {}", self.method_id);
//           }
//
//         }
//       }
//       _ => {
//         panic!("Unknown class_id {} {}", self.class_id, self.method_id);
//       }
//     }
//
//     Ok(args)
//     // buff.read_byte()
//   }
// }

// pub fn encode_frame() -> Result<Vec<u8>> {
// }
