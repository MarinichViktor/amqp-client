use std::net::TcpStream;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::bail;
use crate::{Result,Error};
use log::{info};
use amqp_protocol::dec::Decode;
use amqp_protocol::enc::Encode;
use crate::protocol::frame::{AmqFrame, AmqMethodFrame};

pub struct AmqpStream {
  pub reader: Arc<Mutex<AmqpStreamReader>>,
  pub writer: Arc<Mutex<AmqpStreamWriter>>,
}

impl AmqpStream {
  pub fn new(url: String) -> Self {
    // todo: clone or investigate
    info!("Connecting to {}", url);
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
  pub fn next_method_frame(&mut self) -> Result<AmqMethodFrame> {
    let mut frame = self.next_frame()?;

    loop {
      match frame {
        AmqFrame::Method(method)  => {
          return Ok(method);
        }
        _ => {
          frame = self.next_frame()?;
        }
      }
    }
  }

  pub fn next_frame(&mut self) -> Result<AmqFrame> {
    info!("Reading next frame");
    let mut frame_header = self.read_cursor(7)?;
    info!("Processing frame header");
    let frame_type = frame_header.read_byte()?;
    let chan = frame_header.read_short()?;
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
        AmqFrame::Method(AmqMethodFrame { chan, class_id, method_id, body })
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

  fn read(&mut self, size: usize) -> Result<Vec<u8>> {
    let mut buff = vec![0_u8;size];
    self.0.read_exact(&mut buff)?;
    Ok(buff)
  }
}
