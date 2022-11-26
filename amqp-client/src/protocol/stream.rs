use std::net::TcpStream;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::bail;
use crate::response;
use log::{info};
use amqp_protocol::dec::Decode;
use amqp_protocol::enc::Encode;
use crate::protocol::frame::{Frame, Method, MethodFrame};
use crate::protocol::methods::connection::{CLASS_CONNECTION, METHOD_START, METHOD_STARTOK, METHOD_TUNE, METHOD_TUNEOK};
// static PROTOCOL_HEADER: [u8;8] = [65,77,81,80,0,0,9,1];

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
  pub fn next_method_frame(&mut self) -> response::Result<MethodFrame> {
    let frame_descriptor = self.next_frame()?;
    let frame;

    loop {
      match frame_descriptor {
        Frame::Method(body) => {
          frame = body;
          break;
        }
        // _ => {
        //   frame_descriptor = self.next_frame()?;
        // }
      }
    }

    Ok(frame)
  }

  pub fn next_frame(&mut self) -> response::Result<Frame> {
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
          CLASS_CONNECTION => {
            let method = match method_id {
              METHOD_START => {
                Method::ConnStart(body.try_into()?)
              }
              METHOD_TUNE => {
                Method::ConnTune(body.try_into()?)
              }
              _ => {
                panic!("unsupporetd method id, {}", method_id);
              }
            };

            Frame::Method(MethodFrame { chan: channel, payload: method })
          }
          _ => {
            panic!("unsupporetd class id, {}", class_id);
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

// impl AmqpStream {
//   pub fn new(connection_opts: ConnectionOpts) -> Self {
//     // todo: clone or investigate
//     let url = format!("{}:{}", connection_opts.host, connection_opts.port);
//     info!("Connecting to {}", url);
//     let tcp_stream = TcpStream::connect(url).unwrap();
//     let tcp_stream2 = tcp_stream.try_clone().unwrap();
//
//     AmqpStream {
//       reader: Arc::new(Mutex::new(AmqpStreamReader(tcp_stream))),
//       writer: Arc::new(Mutex::new(AmqpStreamWriter(tcp_stream2))),
//       connection_opts
//     }
//   }

  // pub fn invoke<T: TryInto<Vec<u8>, Error=response::Error>>(&mut self, chan: i16, args: T) -> response::Result<()> {
  //   let mut frame_buff = vec![];
  //   frame_buff.write_byte(1)?;
  //   frame_buff.write_short(chan)?;
  //
  //   let arg_buff = args.try_into()?;
  //   Encode::write_uint(&mut frame_buff, arg_buff.len() as u32)?;
  //   frame_buff.write(&arg_buff)?;
  //   frame_buff.write_byte(0xCE)?;
  //   self.send_raw(&frame_buff)?;
  //
  //   Ok(())
  // }
  //
  // pub fn send_raw(&mut self, buff: &[u8]) -> response::Result<()> {
  //   self.tcp_stream.write_all(buff)?;
  //   Ok(())
  // }

  // pub fn next_method_frame(&mut self) -> response::Result<MethodFrame> {
  //   let frame_descriptor = self.next_frame()?;
  //   let frame;
  //
  //   loop {
  //     match frame_descriptor {
  //       Frame::Method(body) => {
  //         frame = body;
  //         break;
  //       }
  //       // _ => {
  //       //   frame_descriptor = self.next_frame()?;
  //       // }
  //     }
  //   }
  //
  //   Ok(frame)
  // }
  //
  // pub fn next_frame(&mut self) -> response::Result<Frame> {
  //   info!("Reading next frame");
  //   let mut frame_header = self.read_cursor(7)?;
  //   info!("Processing frame header");
  //   let frame_type = frame_header.read_byte()?;
  //   let channel = frame_header.read_short()?;
  //   let size = Decode::read_int(&mut frame_header)?;
  //   info!("Next frame size {}", size);
  //   let body = self.read(size as usize)?;
  //   // read frame end byte
  //   self.read(1)?;
  //   info!("Frame readed");
  //
  //   let frame = match frame_type {
  //     1 => {
  //       let mut meta = Cursor::new(body[..4].to_vec());
  //       let class_id = meta.read_short()?;
  //       let method_id = meta.read_short()?;
  //
  //       match class_id {
  //         CLASS_CONNECTION => {
  //           let method = match method_id {
  //             METHOD_START => {
  //               Method::ConnStart(body.try_into()?)
  //             }
  //             METHOD_TUNE => {
  //               Method::ConnTune(body.try_into()?)
  //             }
  //             _ => {
  //               panic!("unsupporetd method id, {}", method_id);
  //             }
  //           };
  //
  //           Frame::Method(MethodFrame { chan: channel, payload: method })
  //         }
  //         _ => {
  //           panic!("unsupporetd class id, {}", class_id);
  //         }
  //       }
  //     },
  //     // todo: fix this
  //     _ => {
  //       panic!("Unknown frame type")
  //     }
  //   };
  //
  //   Ok(frame)
  // }
  //
  // fn read_cursor(&mut self, size: usize) -> response::Result<Cursor<Vec<u8>>> {
  //   let mut buff = vec![0_u8;size];
  //   info!("Starting read exact {}", size);
  //   match self.tcp_stream.read_exact(&mut buff) {
  //     Ok(_) => {},
  //     Err(e) => {
  //       println!("error ouccured {}", e.kind());
  //       bail!(e.to_string());
  //     }
  //   };
  //
  //   info!("Finished read exact");
  //   Ok(Cursor::new(buff))
  // }
  //
  // fn read(&mut self, size: usize) -> response::Result<Vec<u8>> {
  //   let mut buff = vec![0_u8;size];
  //   self.tcp_stream.read_exact(&mut buff)?;
  //   Ok(buff)
  // }
// }
