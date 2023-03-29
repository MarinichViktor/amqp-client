use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedWriteHalf};
use amqp_protocol::enc::Encode;
use amqp_protocol::types::AmqpMethodArgs;
use crate::{Result};
use crate::protocol::frame2::{Frame2, RawFrame};
use crate::protocol::frame::Frame;

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

  pub async fn write(&mut self, frame: RawFrame) -> Result<()> {
    let buf: Vec<Vec<u8>> = frame.into();

    for amqp_frame in buf {
      self.write_all(& amqp_frame).await?;
    }

    Ok(())
  }

  pub async fn write2<T: AmqpMethodArgs>(&mut self, frame: Frame2<T>) -> Result<()> {
    self.write(frame.into()).await
  }
}
