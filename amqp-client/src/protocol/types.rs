use std::collections::HashMap;

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
