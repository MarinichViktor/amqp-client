use std::io::Cursor;
use anyhow::bail;
use bytes::{Buf, BytesMut};
use log::debug;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use amqp_protocol::dec::Decode;
use crate::protocol::frame::{BodyFrame, Frame, HeaderFrame, MethodFrame};
use crate::{Result, Connection};

const FRAME_HEADER_SIZE: usize = 7;
const FRAME_END_SIZE: usize = 1;

pub struct FrameReader {
  inner: BufReader<OwnedReadHalf>,
  buf: BytesMut,
}

impl FrameReader {
  pub fn new(inner: BufReader<OwnedReadHalf>) -> Self {
    Self {
      inner,
      // todo: review default capacity
      buf: BytesMut::with_capacity(128 * 1024)
    }
  }

  pub async fn next_method_frame(&mut self) -> Result<MethodFrame> {
    let mut frame = self.next_frame().await?;

    loop {
      match frame {
        Frame::Method(method)  => {
          return Ok(method);
        },
        // todo: check for the header frame?
        _ => {
          frame = self.next_frame().await?;
        }
      }
    }
  }

  pub async fn next_frame(&mut self) -> Result<Frame> {
    loop {
      if let Some(frame) = self.read_frame()? {
        return Ok(frame);
      }

      // todo: what if no capacity left?
      if 0 == self.inner.read_buf(&mut self.buf).await? {
        // todo: add check for size of the buf, if its error or connection close
        bail!("Failed to read. Connection closed")
      }
    }
  }

  pub fn read_frame(&mut self) -> Result<Option<Frame>> {
    if self.buf.len() < FRAME_HEADER_SIZE {
      debug!("read_frame: frame header is no available");
      return Ok(None);
    }

    let mut buf = Cursor::new(&self.buf[..7]);
    let frame_type = buf.read_byte()?;
    let chan = buf.read_short()?;
    let size = buf.read_int()?;

    // header + body_size + frame_end_byte
    let frame_size = FRAME_HEADER_SIZE + size as usize + FRAME_END_SIZE;
    if self.buf.len() < frame_size as usize {
      debug!("read_frame: frame body is no available");
      return Ok(None)
    }

    self.buf.advance(7);
    let body = self.buf.split_to(size as usize).to_vec();
    // read frame end byte
    assert_eq!(206, self.buf[0]);
    self.buf.advance(1);

    debug!("read_frame: Type {}, Chan {}, Size {}", frame_type, chan, size);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::Method(MethodFrame { chan, class_id, method_id, body, content: None })
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

    Ok(Some(frame))
  }
}
