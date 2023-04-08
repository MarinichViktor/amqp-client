use std::io::Write;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedWriteHalf};
use crate::protocol::enc::Encode;
use crate::protocol::types::AmqMethod;
use crate::{Result};

pub struct ConWriter {
  inner: BufWriter<OwnedWriteHalf>
}

impl ConWriter {
  pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
    Self { inner }
  }

  pub async fn invoke(&mut self, ch: i16, args: impl AmqMethod) -> Result<()> {
    let raw_frame = Self::build_frame(ch, args, None)?;
    self.write_bytes(&raw_frame).await?;
    Ok(())
  }

  pub async fn write_bytes<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
    self.inner.write_all(buf).await?;
    self.inner.flush().await?;
    Ok(())
  }

  // todo: move outside
  pub fn build_frame(ch: i16, args: impl AmqMethod, payload: Option<u8>) -> Result<Vec<u8>> {
    let args = args.into_raw();

    let mut frame_buff = vec![];
    frame_buff.write_byte(1)?;
    frame_buff.write_short(ch)?;
    frame_buff.write_uint(args.len() as u32)?;
    Write::write(&mut frame_buff, &args)?;
    frame_buff.write_byte(0xCE)?;

    Ok(frame_buff)
  }

  pub async fn write_method_frame(&mut self, chan: i16, args: Vec<u8>, body: Option<Vec<u8>>) -> Result<()> {
    use std::io::Write;

    let arg_buff = args;
    let mut frame_buff = vec![];
    frame_buff.write_byte(1)?;
    frame_buff.write_short(chan)?;
    Encode::write_uint(&mut frame_buff, arg_buff.len() as u32)?;
    Write::write(&mut frame_buff, &arg_buff)?;
    frame_buff.write_byte(0xCE)?;
    self.write_bytes(&frame_buff).await?;

    if let Some(mut body) = body {
      let mut raw_header = vec![];
      raw_header.write_short(60)?;
      raw_header.write_short(0)?;
      raw_header.write_long(body.len() as i64)?;
      raw_header.write_short(0)?;

      let mut raw_frame = vec![];
      raw_frame.write_byte(2)?;
      raw_frame.write_short(chan)?;
      raw_frame.write_int(raw_header.len() as i32)?;
      raw_frame.append(&mut raw_header);
      raw_frame.write_byte(0xCE)?;

      self.write_bytes(&raw_frame).await?;

      let mut raw_frame = vec![];
      raw_frame.write_byte(3)?;
      raw_frame.write_short(chan)?;
      raw_frame.write_int(body.len() as i32)?;
      raw_frame.append(&mut body);
      raw_frame.write_byte(0xCE)?;
      self.write_bytes(&raw_frame).await?;
    }

    Ok(())
  }
}
