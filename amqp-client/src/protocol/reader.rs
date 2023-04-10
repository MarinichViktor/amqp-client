use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use anyhow::bail;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use crate::protocol::dec::Decode;
use crate::{Result};
use crate::protocol::types::{ChannelId, ContentBody, ContentHeader, Frame};

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
      buf: BytesMut::with_capacity(128 * 1024),
    }
  }

  pub async fn next_frame(&mut self) -> Result<(ChannelId, Frame)> {
    loop {
      if let Some(amqp_frame) = self.parse_frame()? {
        return Ok(amqp_frame);
        // let result = match amqp_frame {
        //   Frame::Method(method) => {
        //     if !method.has_content() {
        //       let raw_frame = RawFrame::new(method.chan, method.class_id, method.method_id, method.body, None, None);
        //       Some(FrameKind::Method(raw_frame))
        //     } else {
        //       self.pending_frames.lock().unwrap().insert(method.chan, PendingFrame::new(method));
        //       None
        //     }
        //   }
        //   Frame::Header(header) => {
        //     let ch = header.chan;
        //     let mut pending = self.pending_frames.lock().unwrap();
        //     pending.get_mut(&ch).unwrap().header(header);
        //     None
        //   },
        //   Frame::Body(mut frame) => {
        //     let mut pending_frames = self.pending_frames.lock().unwrap();
        //     let mut pending_frame = pending_frames.remove(&frame.chan).unwrap();
        //     pending_frame.append_body(&mut frame.body);
        //
        //     if pending_frame.is_completed() {
        //       Some(FrameKind::Method(pending_frame.into()))
        //     } else {
        //       pending_frames.insert(frame.chan, pending_frame);
        //       None
        //     }
        //   },
        //   Frame::Heartbeat => {
        //     Some(FrameKind::Heartbeat)
        //   }
        // };
        //
        // if let Some(frame) = result {
        //   return Ok(frame);
        // };
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

  fn parse_frame(&mut self) -> Result<Option<(ChannelId, Frame)>> {
    if !self.has_frame()? {
      return Ok(None);
    }

    let header = self.buf.split_to(7);
    let mut header = Cursor::new(&header[..]);
    let frame_type = header.read_byte()?;
    let chan = header.read_short()?;
    let size = header.read_int()?;

    let body = self.buf.split_to(size as usize).to_vec();
    // read frame end byte
    assert_eq!(206, self.buf[0]);
    self.buf.advance(1);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::method(class_id, method_id, &body)
      },
      2 => {
        let mut meta = Cursor::new(body[..12].to_vec());
        let class_id = meta.read_short()?;
        let _weight = meta.read_short()?;
        // todo: review type
        let body_len = meta.read_long()?;

        // Frame::Header(HeaderFrame {
        //   chan,
        //   class_id,
        //   body_len,
        //   prop_list: body[12..].to_vec()
        // })

        Frame::ContentHeader(ContentHeader {
          class_id,
          body_len: body_len as u64,
          prop_list: body[12..].to_vec().into()
        })
      }
      3 => {
        Frame::ContentBody(ContentBody(body))
      }
      8 => {
        Frame::Heartbeat
      },
      _ => {
        panic!("Unexpected frame")
      }
    };

    Ok(Some((chan, frame)))
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
