use std::net::TcpStream;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::bail;
use crate::{Result,Error};
use log::{debug, info};
use amqp_protocol::dec::Decode;
use amqp_protocol::enc::Encode;
use crate::protocol::frame::{BodyFrame, Frame, HeaderFrame, MethodFrame};

pub struct AmqpStream {
  pub reader: Arc<Mutex<AmqpStreamReader>>,
  pub writer: Arc<Mutex<AmqpStreamWriter>>,
}

impl AmqpStream {
  pub fn new(url: String) -> Self {
    // todo: clone or investigate
    debug!("Connecting to {}", url);
    let tcp_stream = TcpStream::connect(url).unwrap();
    let tcp_stream2 = tcp_stream.try_clone().unwrap();

    AmqpStream {
      reader: Arc::new(Mutex::new(AmqpStreamReader(tcp_stream))),
      writer: Arc::new(Mutex::new(AmqpStreamWriter(tcp_stream2))),
    }
  }
}

pub struct AmqpStreamWriter(TcpStream);

impl AmqpStreamWriter {
  pub fn invoke<T: TryInto<Vec<u8>, Error=Error>>(&mut self, chan: i16, args: T) -> Result<()> {
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

  pub fn send_raw(&mut self, buff: &[u8]) -> Result<()> {
    self.0.write_all(buff)?;
    Ok(())
  }
}

pub struct AmqpStreamReader(TcpStream);

impl AmqpStreamReader {
  // todo: check channel
  pub fn next_method_frame(&mut self) -> Result<MethodFrame> {
    let mut frame = self.next_frame()?;

    loop {
      match frame {
        Frame::Method(method)  => {
          return Ok(method);
        },
        // AmqFrame::Header(header) => {
        //
        // }
        _ => {
          frame = self.next_frame()?;
        }
      }
    }
  }

  pub fn next_frame(&mut self) -> Result<Frame> {
    let mut frame_header = self.read_cursor(7)?;
    let frame_type = frame_header.read_byte()?;
    let chan = frame_header.read_short()?;
    let size = Decode::read_int(&mut frame_header)?;
    let body = self.read(size as usize)?;
    // read frame end byte
    self.read(1)?;
    info!("next_frame frame_type {}, chan {}, size {}, body {:?}", frame_type, chan, size, body);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::Method(MethodFrame { chan, class_id, method_id, body, content_header: None, content_body: None })
      },
      2 => {
        let mut meta = Cursor::new(body[..14].to_vec());
        let class_id = meta.read_short()?;
        let _weight = meta.read_short()?;
        let body_len = meta.read_long()?;
        let prop_flags = meta.read_short()?;

        Frame::Header(HeaderFrame {
          chan,
          class_id,
          body_len,
          prop_flags,
          prop_list: body[14..].to_vec()
        })
      }
      3 => {
        info!("Body frame {:?}, len {}", &body, body.len());
        Frame::Body(BodyFrame {
          chan,
          body
        })
      }
      4 => {
        Frame::Heartbeat
      },
      // todo: fix this
      _ => {
        panic!("Unknown frame type")
      }
    };

    Ok(frame)
  }

  fn read_cursor(&mut self, size: usize) -> Result<Cursor<Vec<u8>>> {
    let mut buff = vec![0_u8;size];
    match self.0.read_exact(&mut buff) {
      Ok(_) => {},
      Err(e) => {
        println!("error ouccured {}", e.kind());
        bail!(e.to_string());
      }
    };

    Ok(Cursor::new(buff))
  }

  fn read(&mut self, size: usize) -> Result<Vec<u8>> {
    let mut buff = vec![0_u8;size];
    self.0.read_exact(&mut buff)?;
    Ok(buff)
  }
}
