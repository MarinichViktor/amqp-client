use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedWriteHalf};
use amqp_protocol::enc::Encode;
use crate::{Result};

pub struct FrameWriter {
  inner: BufWriter<OwnedWriteHalf>
}

impl FrameWriter {
  pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
    Self { inner }
  }

  pub async fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
    self.inner.write_all(buf).await?;
    self.inner.flush().await?;
    Ok(())
  }

  pub async fn write_method_frame(&mut self, chan: i16, args: Vec<u8>) -> Result<()> {
    use std::io::Write;

    let arg_buff = args;
    let mut frame_buff = vec![];
    frame_buff.write_byte(1)?;
    frame_buff.write_short(chan)?;
    Encode::write_uint(&mut frame_buff, arg_buff.len() as u32)?;
    Write::write(&mut frame_buff, &arg_buff)?;
    frame_buff.write_byte(0xCE)?;
    self.write_all(&frame_buff).await?;

    Ok(())
  }
}
