use std::net::TcpStream;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::bail;
use crate::response;
use log::{info};
use amqp_protocol::dec::Decode;
use amqp_protocol::enc::Encode;
use crate::protocol::frame::{AmqpFrame, AmqpFrameType, Method};
use crate::protocol::methods::{connection as conn_methods, channel as chan_methods};

pub struct AmqpStream {
  pub reader: Arc<Mutex<AmqpStreamReader>>,
  pub writer: Arc<Mutex<AmqpStreamWriter>>,
  connection_opts: ConnectionOpts
}

impl AmqpStream {
  pub fn new(connection_opts: ConnectionOpts) -> Self {
    // todo: clone or investigate
    let url = format!("{}:{}", connection_opts.host, connection_opts.port);
    info!("Connecting to {}", url);
    let tcp_stream = TcpStream::connect(url).unwrap();
    let tcp_stream2 = tcp_stream.try_clone().unwrap();

    AmqpStream {
      reader: Arc::new(Mutex::new(AmqpStreamReader(tcp_stream))),
      writer: Arc::new(Mutex::new(AmqpStreamWriter(tcp_stream2))),
      connection_opts
    }
  }
}

pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String
}

pub struct AmqpStreamWriter(TcpStream);

impl AmqpStreamWriter {
  pub fn invoke<T: TryInto<Vec<u8>, Error=response::Error>>(&mut self, chan: i16, args: T) -> response::Result<()> {
    let mut frame_buff = vec![];
    frame_buff.write_byte(1)?;
    frame_buff.write_short(chan)?;

    let arg_buff = args.try_into()?;
    Encode::write_uint(&mut frame_buff, arg_buff.len() as u32)?;
    frame_buff.write(&arg_buff)?;
    frame_buff.write_byte(0xCE)?;
    self.send_raw(&frame_buff)?;

    Ok(())
  }

  pub fn send_raw(&mut self, buff: &[u8]) -> response::Result<()> {
    self.0.write_all(buff)?;
    Ok(())
  }
}

pub struct AmqpStreamReader(TcpStream);

impl AmqpStreamReader {
  pub fn next_method_frame(&mut self) -> response::Result<AmqpFrame> {
    let mut frame_descriptor = self.next_frame()?;

    loop {
      match frame_descriptor.ty {
        AmqpFrameType::Method  => {
          return Ok(frame_descriptor);
        }
        _ => {
          frame_descriptor = self.next_frame()?;
        }
      }
    }
  }

  pub fn next_frame(&mut self) -> response::Result<AmqpFrame> {
    info!("Reading next frame");
    let mut frame_header = self.read_cursor(7)?;
    info!("Processing frame header");
    let frame_type = frame_header.read_byte()?;
    let channel = frame_header.read_short()?;
    let size = Decode::read_int(&mut frame_header)?;
    info!("Next frame size {}", size);
    let body = self.read(size as usize)?;
    // read frame end byte
    self.read(1)?;
    info!("Frame readed");

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        match class_id {
          conn_methods::CLASS_CONNECTION => {
            let method = match method_id {
              conn_methods::METHOD_START => {
                Method::ConnStart(body.try_into()?)
              }
              conn_methods::METHOD_TUNE => {
                Method::ConnTune(body.try_into()?)
              }
              conn_methods::METHOD_OPENOK => {
                Method::ConnOpenOk(body.try_into()?)
              }
              _ => {
                panic!("unsupporetd method id, class_id: {}, method_id: {}", class_id, method_id);
              }
            };
            AmqpFrame::method(channel, method)
          },
          chan_methods::CLASS_CHANNEL => {
            let method = match method_id {
              chan_methods::METHOD_OPEN_OK => {
                Method::ChanOpenOk(body.try_into()?)
              }
              _ => {
                panic!("unsupported method id, {}", method_id);
              }
            };
            AmqpFrame::method(channel, method)
          }
          _ => {
            panic!("unsupported class id, class_id: {}, method_id: {}", class_id, method_id);
          }
        }
      },
      // todo: fix this
      _ => {
        panic!("Unknown frame type")
      }
    };

    Ok(frame)
  }

  fn read_cursor(&mut self, size: usize) -> response::Result<Cursor<Vec<u8>>> {
    let mut buff = vec![0_u8;size];
    info!("Starting read exact {}", size);
    match self.0.read_exact(&mut buff) {
      Ok(_) => {},
      Err(e) => {
        println!("error ouccured {}", e.kind());
        bail!(e.to_string());
      }
    };

    info!("Finished read exact");
    Ok(Cursor::new(buff))
  }

  fn read(&mut self, size: usize) -> response::Result<Vec<u8>> {
    let mut buff = vec![0_u8;size];
    self.0.read_exact(&mut buff)?;
    Ok(buff)
  }
}
