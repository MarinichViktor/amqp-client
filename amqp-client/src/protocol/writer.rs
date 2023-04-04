use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedWriteHalf};
use amqp_protocol::types::AmqpMethodArgs;
use crate::{Result};
use crate::protocol::frame2::{RawFrame};

pub struct FrameWriter {
  inner: BufWriter<OwnedWriteHalf>
}

impl FrameWriter {
  pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
    Self { inner }
  }

  pub async fn write_binary<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
    self.inner.write_all(buf).await?;
    self.inner.flush().await?;
    Ok(())
  }

  pub async fn send_raw_frame(&mut self, frame: RawFrame) -> Result<()> {
    let buf: Vec<Vec<u8>> = frame.into();

    for amqp_frame in buf {
      self.write_binary(& amqp_frame).await?;
    }

    Ok(())
  }

  pub async fn send_method<T: AmqpMethodArgs>(&mut self, ch: i16, args: T) -> Result<()> {
    let raw_frame = RawFrame::new(
      ch,
      args.class_id(),
      args.method_id(),
      args.try_into()?,
      None,
      None
    );
    self.send_raw_frame(raw_frame).await
  }

}
