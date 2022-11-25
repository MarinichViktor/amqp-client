use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Cursor, Read, Write};
use crate::response;
use super::{frame::{Frame}};
// use byteorder::{BigEndian, ReadBytesExt};
use crate::protocol::decoder::Decode;
use log::{info};
// use crate::protocol::encoder::Encode;
// use crate::protocol::frame::encoding::encode_method_frame;
use crate::protocol::frame::{ConnectionMethod, MethodFrame};
use crate::protocol::method::{ServerProperty};

static PROTOCOL_HEADER: [u8;8] = [65,77,81,80,0,0,9,1];

pub struct AmqpStream {
  tcp_stream: TcpStream,
  connection_opts: ConnectionOpts
}

pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String
}

impl AmqpStream {
  pub fn new(connection_opts: ConnectionOpts) -> Self {
    // todo: clone or investigate
    let url = format!("{}:{}", connection_opts.host, connection_opts.port);
    info!("Connecting to {}", url);
    let tcp_stream = TcpStream::connect(url).unwrap();

    AmqpStream {
      tcp_stream,
      connection_opts
    }
  }

  pub fn protocol_header(& mut self) -> response::Result<()> {
    self.tcp_stream.write_all(&PROTOCOL_HEADER)?;
    Ok(())
  }

  pub fn start_ok(
    &mut self,
    client_properties: HashMap<String, ServerProperty>,
    mechanism: String,
    response: String,
    locale: String
  ) -> response::Result<()> {
    let mut args = vec![
      ServerProperty::PropTable(client_properties),
      ServerProperty::ShortStr(mechanism),
      ServerProperty::LongStr(response),
      ServerProperty::ShortStr(locale),
    ];

    // let raw_frame = encode_method_frame(0, args)?;
    // self.tcp_stream.write_all(&raw_frame)?;

    Ok(())
  }

  pub fn next_method_frame(&mut self) -> response::Result<MethodFrame> {
    let mut frame_descriptor = self.next_frame()?;
    let mut frame;

    loop {
      match frame_descriptor {
        Frame::Method(body) => {
          frame = body;
          break;
        }
        _ => {
          frame_descriptor = self.next_frame()?;
        }
      }
    }

    Ok(frame)
  }

  pub fn next_frame(&mut self) -> response::Result<Frame> {
    let mut frame_header = self.read_cursor(7)?;
    let frame_type = frame_header.read_byte()?;
    let channel = frame_header.read_short()?;
    let size = Decode::read_int(&mut frame_header)?;
    let mut body = self.read(size as usize)?;
    // read frame end byte
    self.read(1)?;

    match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let mut payload = Cursor::new(body[4..].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        match class_id {
          10 => {
            match method_id {
              10 => {
                return Ok(
                  Frame::Method(
                    MethodFrame::Connection(
                      ConnectionMethod::Start {
                        channel,
                        ver_major: payload.read_byte()?,
                        ver_minor: payload.read_byte()?,
                        properties: payload.read_prop_table()?,
                        mechanisms: payload.read_long_str()?.split(' ').map( |s| s.into()).collect(),
                        locales: payload.read_long_str()?.split(' ').map( |s| s.into()).collect()
                      }
                    )
                  )
                )
              }
              _ => {
                panic!("unsupporetd method id");
              }
            }
          }
          _ => {
            panic!("unsupporetd class id");
          }
        }
      },
      // todo: fix this
      _ => {
        panic!("Unknown frame type")
      }
    };
  }

  fn read_cursor(&mut self, size: usize) -> response::Result<Cursor<Vec<u8>>> {
    let mut buff = vec![0_u8;size];
    self.tcp_stream.read_exact(&mut buff)?;
    Ok(Cursor::new(buff))
  }

  fn read(&mut self, size: usize) -> response::Result<Vec<u8>> {
    let mut buff = vec![0_u8;size];
    self.tcp_stream.read_exact(&mut buff)?;
    Ok(buff)
  }
}
