use std::collections::HashMap;
use crate::{define_amqp_classes};

pub type PropTable = HashMap<ShortStr, Property>;

pub type Byte = u8;
pub type Bool = bool;
pub type UShort = u16;
pub type Short = i16;
pub type Int = i32;
pub type UInt = u32;
pub type Long = i64;
pub type ULong = u64;
pub type Float = f32;
pub type Double = f64;
pub type ChannelId = i16;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortStr(pub String);

impl From<String> for ShortStr {
  fn from(str: String) -> Self {
    Self(str)
  }
}

impl From<&str> for ShortStr {
  fn from(str: &str) -> Self {
    Self(str.into())
  }
}

#[derive(Default, Debug, Clone)]
pub struct LongStr(pub String);

impl From<String> for LongStr {
  fn from(str: String) -> Self {
    Self(str)
  }
}

impl From<&str> for LongStr {
  fn from(str: &str) -> Self {
    Self(str.into())
  }
}

#[derive(Debug, Clone)]
pub enum Property {
  Bool(bool),
  Byte(u8),
  Short(i16),
  UShort(u16),
  Int(i32),
  UInt(u32),
  Long(i64),
  ULong(u64),
  Float(f32),
  Double(f64),
  ShortStr(ShortStr),
  LongStr(LongStr),
  Table(PropTable)
}

pub trait AmqpMethodArgs: TryInto<Vec<u8>, Error=crate::Error> + TryFrom<Vec<u8>, Error=crate::Error> {
  fn class_id(&self) -> i16;
  fn method_id(&self) -> i16;
}
use paste::paste;
use crate::api::basic::fields::MessageProperties;
use crate::protocol::dec::Decode;
use crate::protocol::enc::Encode;

define_amqp_classes! {
  Connection(10) {
    Start(10) { ver_major: Byte, ver_minor: Byte, properties: PropTable, mechanisms: LongStr, locales: LongStr, }
    StartOk(11) { properties: PropTable, mechanism: ShortStr, response: LongStr, locale: ShortStr, }
    Tune(30) { chan_max: Short, frame_max: Int, heartbeat: Short, }
    TuneOk(31) { chan_max: Short, frame_max: Int, heartbeat: Short, }
    Open(40) { vhost: ShortStr, reserved1: ShortStr, reserved2: Byte, }
    OpenOk(41) { reserved1: ShortStr, }
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
    Deliver(60) { consumer_tag: ShortStr, deliver_tag: ULong, redelivered: Byte, exhchange: ShortStr, routing_key: ShortStr, }
  }
}

pub type AmqpMessage = (ChannelId, Frame);

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
