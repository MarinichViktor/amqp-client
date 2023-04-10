use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedWriteHalf};
use crate::protocol::types::{ChannelId, Frame};
use crate::{Result};
use crate::protocol::enc::Encode;

pub struct FrameWriter {
  inner: BufWriter<OwnedWriteHalf>
}

impl FrameWriter {
  pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
    Self { inner }
  }

  pub async fn dispatch(&mut self, channel: ChannelId, frame: Frame) -> Result<()> {
    let frame_ty = match &frame {
      Frame::ContentHeader(..) => 2,
      Frame::ContentBody(..) => 3,
      Frame::Heartbeat => 8,
      _ => 1,
    };

    let mut payload = frame.to_raw_repr();
    let mut frame_buff = vec![];

    frame_buff.write_byte(frame_ty).unwrap();
    frame_buff.write_short(channel).unwrap();
    frame_buff.write_uint(payload.len() as u32).unwrap();
    frame_buff.append(&mut payload);
    frame_buff.write_byte(0xCE).unwrap();

    self.write_binary(&frame_buff).await?;

    Ok(())
  }

  pub async fn write_binary<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
    self.inner.write_all(buf).await?;
    self.inner.flush().await?;
    Ok(())
  }
}
