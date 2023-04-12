use crate::{generate_protocol_methods};

use paste::paste;
use crate::protocol::dec::Decode;
use crate::protocol::enc::Encode;
use crate::protocol::message::MessageProperties;
use crate::protocol::types::{Bool, ChannelId, Long};
use super::types::{Byte, PropTable, LongStr, ShortStr, Short, Int, ULong};

generate_protocol_methods! {
  Connection(10) {
    Start(10) { ver_major: Byte, ver_minor: Byte, properties: PropTable, mechanisms: LongStr, locales: LongStr, }
    StartOk(11) { properties: PropTable, mechanism: ShortStr, response: LongStr, locale: ShortStr, }
    Tune(30) { chan_max: Short, frame_max: Int, heartbeat: Short, }
    TuneOk(31) { chan_max: Short, frame_max: Int, heartbeat: Short, }
    Open(40) { vhost: ShortStr, reserved1: ShortStr, reserved2: Byte, }
    OpenOk(41) { reserved1: ShortStr, }
    Close(50) { reply_code: Short, reply_text: ShortStr, class_id: Short, method_id: Short, }
    CloseOk(50) { }
  }
  Channel(20) {
    Open(10) { reserved1: ShortStr, }
    OpenOk(11) { reserved1: ShortStr, }
    Flow(20) { active: Byte, }
    FlowOk(21) { active: Byte, }
    Close(40) { reply_code: Short, reply_text: ShortStr, class_id: Short, method_id: Short, }
    CloseOk(40) { }
  }
  Exchange(40) {
    Declare(10) { reserved1: Short, name: ShortStr, ty: ShortStr, flags: Byte, props: PropTable, }
    DeclareOk(11) { }
    Delete(20) { reserved1: ShortStr, name: ShortStr, del_if_unused: Byte, no_wait: Byte, }
    DeleteOk(21) { }
  }
  Queue(50) {
    Declare(10) { reserved1: Short, name: ShortStr, flags: Byte, props: PropTable, }
    DeclareOk(11) { name: ShortStr, msg_count: Int, consumer_count: Int, }
    Bind(20) { reserved1: Short, queue: ShortStr, exchange: ShortStr, routing_key: ShortStr, no_wait: Byte, table: PropTable, }
    BindOk(21) { }
    Unbind(50) { reserved1: Short, queue: ShortStr, exchange: ShortStr, routing_key: ShortStr, table: PropTable, }
    UnbindOk(51) { }
  }
  Basic(60) {
    Consume(20) { reserved1: Short, queue: ShortStr, tag: ShortStr, flags: Byte, props: PropTable, }
    ConsumeOk(21) { tag: ShortStr, }
    Publish(40) { reserved1: Short, exchange: ShortStr, routing_key: ShortStr, flags: Byte, }
    Deliver(60) { consumer_tag: ShortStr, deliver_tag: Long, redelivered: Bool, exchange: ShortStr, routing_key: ShortStr, }
    Ack(80) { delivery_tag: Long, multiple: Bool, }
    Reject(90) { delivery_tag: Long, requeue: Bool, }
  }
}

#[derive(Debug)]
pub struct ContentHeader {
  pub class_id: Short,
  pub body_len: Long,
  pub prop_list: MessageProperties,
}

impl ContentHeader {
  pub fn from_raw_repr(mut buf: &[u8]) -> Self {
    let class_id = buf.read_short().unwrap();
    buf.read_short().unwrap();
    let body_len = buf.read_long().unwrap();

    Self {
      class_id,
      body_len,
      prop_list: buf[12..].to_vec().into()
    }
  }

  pub fn to_raw_repr(self) -> Vec<u8> {
    let mut buf = vec![];
    buf.write_short(self.class_id).unwrap();
    buf.write_short(0).unwrap();
    buf.write_long(self.body_len as Long).unwrap();
    buf.append(&mut self.prop_list.into());
    buf
  }

  pub fn into_frame(self) -> Frame {
    Frame::ContentHeader(self)
  }
}

#[derive(Debug)]
pub struct ContentBody(pub Vec<u8>);

impl ContentBody {
  pub fn from_raw_repr(mut buf: &[u8]) -> Self {
    Self(buf.to_vec())
  }

  pub fn to_raw_repr(self) -> Vec<u8> {
    self.0
  }

  pub fn into_frame(self) -> Frame {
    Frame::ContentBody(self)
  }
}

#[derive(Debug)]
pub enum ContentFrame {
  WithMethod(Frame),
  WithContentHeader((Frame, ContentHeader)),
  WithBody((Frame, ContentHeader, ContentBody))
}


impl ContentFrame {
  fn new(method: Frame) -> Self {
    Self::WithMethod(method)
  }

  pub fn with_content_header(self, header: ContentHeader) -> Self {
    if let ContentFrame::WithMethod(frame) = self {
      Self::WithContentHeader((frame, header))
    } else {
      panic!("Invalid state transition")
    }
  }

  pub fn with_body(self, mut body: ContentBody) -> Self {
    match self {
      ContentFrame::WithContentHeader((frame, header)) => {
        Self::WithBody((frame, header, body))
      },
      ContentFrame::WithBody((frame, header, mut curr_body)) => {
        curr_body.0.append(&mut body.0);
        Self::WithBody((frame, header, curr_body))
      },
      _ => {
        panic!("Invalid state transition")
      }
    }
  }

  pub fn is_complete(&self) -> bool {
    match self {
      ContentFrame::WithBody((_,header,body)) => {
        header.body_len <= body.0.len() as Long
      }
      _ => {
        false
      }
    }
  }
}

pub type FrameEnvelope = (ChannelId, Frame);
