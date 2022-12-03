use std::collections::HashMap;

pub type Table = HashMap<String, Property>;

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
  ShortStr(String),
  LongStr(String),
  Table(Table)
}

pub trait AmqMethod {
  fn class_id(&self) -> i16;
  fn method_id(&self) -> i16;
}
