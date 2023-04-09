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
use crate::protocol::dec::Decode;
use crate::protocol::enc::Encode;

define_amqp_classes! {
  Connection(10) {
    Start(10) { ver_major: Byte, ver_minor: Byte, properties: PropTable, mechanisms: LongStr, locales: LongStr }
    StartOk(11) { properties: PropTable, mechanism: ShortStr, response: LongStr, locale: ShortStr }
    Tune(30) { chan_max: Short, frame_max: Int, heartbeat: Short }
    TuneOk(31) { chan_max: Short, frame_max: Int, heartbeat: Short }
    Open(40) { vhost: ShortStr, reserved1: ShortStr, reserved2: Byte }
    OpenOk(41) { reserved1: ShortStr }
  }
}

