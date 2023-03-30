use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use anyhow::bail;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use amqp_protocol::dec::Decode;
use crate::protocol::frame::{BodyFrame, Frame, HeaderFrame, MethodFrame};
use crate::{Result};
use crate::protocol::frame2::{PendingFrame, RawFrame};

const FRAME_HEADER_SIZE: usize = 7;
const FRAME_END_SIZE: usize = 1;

pub struct FrameReader {
  inner: BufReader<OwnedReadHalf>,
  buf: BytesMut,
  pending_frames: Arc<Mutex<HashMap<i16, PendingFrame>>>
}

impl FrameReader {
  pub fn new(inner: BufReader<OwnedReadHalf>) -> Self {
    Self {
      inner,
      buf: BytesMut::with_capacity(128 * 1024),
      pending_frames: Default::default()
    }
  }

  pub async fn next_frame(&mut self) -> Result<RawFrame> {
    loop {
      if let Some(amqp_frame) = self.parse_frame()? {
        let result = match amqp_frame {
          Frame::Method(method) => {
            if !method.has_content() {
              let raw_frame = RawFrame::new(method.chan, method.class_id, method.method_id, method.body, None, None);
              Some(raw_frame)
            } else {
              self.pending_frames.lock().unwrap().insert(method.chan, PendingFrame::new(method));
              None
            }
          }
          Frame::Header(header) => {
            let ch = header.chan;
            let mut pending = self.pending_frames.lock().unwrap();
            pending.get_mut(&ch).unwrap().header(header);
            None
          },
          Frame::Body(mut frame) => {
            let mut pending_frames = self.pending_frames.lock().unwrap();
            let mut pending_frame = pending_frames.remove(&frame.chan).unwrap();
            pending_frame.append_body(&mut frame.body);

            if pending_frame.is_completed() {
              Some(pending_frame.into())
            } else {
              pending_frames.insert(frame.chan, pending_frame);
              None
            }
          },
          Frame::Heartbeat => {
            panic!("Heartbeat received");
          }
        };

        if let Some(raw_frame) = result {
          return Ok(raw_frame);
        };
      }

      if !self.buf.is_empty() && self.has_frame()? {
        continue
      }

      // todo: what if no capacity left?
      if 0 == self.inner.read_buf(&mut self.buf).await? {
        // todo: add check for size of the buf, if its error or connection close
        bail!("Failed to read. Connection closed")
      }
    }
  }

  fn parse_frame(&mut self) -> Result<Option<Frame>> {
    if !self.has_frame()? {
      return Ok(None);
    }

    let mut buf = Cursor::new(&self.buf[..7]);
    let frame_type = buf.read_byte()?;
    let chan = buf.read_short()?;
    let size = buf.read_int()?;

    self.buf.advance(7);
    let body = self.buf.split_to(size as usize).to_vec();
    // read frame end byte
    assert_eq!(206, self.buf[0]);
    self.buf.advance(1);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::Method(MethodFrame { chan, class_id, method_id, body })
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
          prop_list: body[12..].to_vec()
        })
      }
      3 => {
        Frame::Body(BodyFrame { chan, body })
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


  fn has_frame(&mut self) -> Result<bool> {
    if self.buf.len() < FRAME_HEADER_SIZE {
      return Ok(false);
    }

    let mut buf = Cursor::new(&self.buf[..7]);
    let _frame_type = buf.read_byte()?;
    let _chan = buf.read_short()?;
    let size = buf.read_int()?;

    // header + body_size + frame_end_byte
    let frame_size = FRAME_HEADER_SIZE + size as usize + FRAME_END_SIZE;

    if self.buf.len() < frame_size as usize {
      return Ok(false)
    }

    return Ok(true);
  }
}
