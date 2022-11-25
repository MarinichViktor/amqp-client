use std::collections::HashMap;
use std::io::{Cursor, Write};
use byteorder::ReadBytesExt;
use log::info;
use crate::protocol::decoder::{Decode};
use crate::protocol::encoder::Encode;
use crate::response;

#[derive(Debug)]
pub enum Method {
  StartMethod(StartMethod),
  StartOkMethod(StartOkMethod)
}

pub type PropTable = HashMap<String, ServerProperty>;

#[derive(Debug, Clone)]
pub enum ServerProperty {
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
  PropTable(PropTable)
}

#[derive(Debug)]
pub struct StartMethod {
  pub ver_major: u8,
  pub ver_minor: u8,
  pub server_properties: HashMap<String, ServerProperty>,
  pub mechanisms: Vec<String>,
  pub locales: Vec<String>,
}

impl StartMethod {
  pub fn parse_from<T: AsRef<[u8]>>(orig: T) -> response::Result<StartMethod> {
    let mut reader = Cursor::new(orig.as_ref().to_vec());
    let ver_major = reader.read_u8()?;
    let ver_minor = reader.read_u8()?;
    let server_properties = reader.read_prop_table()?;
    let mechanisms = reader.read_long_str()?.split(' ').map(|x| x.into()).collect();
    let locales = reader.read_long_str()?.split(' ').map(|x| x.into()).collect();

    Ok(StartMethod {
      ver_major,
      ver_minor,
      server_properties,
      mechanisms,
      locales,
    })
  }
}

#[derive(Debug)]
pub struct StartOkMethod {
  pub client_properties: HashMap<String, ServerProperty>,
  pub mechanism: String,
  pub response: String,
  pub locale: String,
}

impl StartOkMethod {
  pub fn to_raw_frame(self) -> response::Result<Vec<u8>> {
    let mut frame = vec![];

    frame.write_byte(1)?;

    frame.write_short(0)?;

    let mut buff = vec![];
    // class id
    buff.write_short(10)?;
    // method id
    buff.write_short(11)?;

    buff.write_prop_table(self.client_properties)?;
    buff.write_short_str(self.mechanism)?;
    buff.write_long_str(self.response)?;
    buff.write_short_str(self.locale)?;

    frame.write_uint(buff.len() as u32)?;
      frame.write(& buff)?;

    frame.write_byte(206 as u8)?;
    // frame.write_byte(0xCE as u8)?;
    Ok(frame)
  }
}
