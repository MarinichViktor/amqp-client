use std::collections::HashMap;
use std::io::{Cursor, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::info;
use crate::protocol::method::ServerProperty;
use crate::response;

pub trait Encode {
  fn write_bool(&mut self, val: bool) -> response::Result<()>;
  fn write_byte(&mut self, val: u8) -> response::Result<()>;
  fn write_short(&mut self, val: i16) -> response::Result<()>;
  fn write_ushort(&mut self, val: u16) -> response::Result<()>;
  fn write_int(&mut self, val: i32) -> response::Result<()>;
  fn write_uint(&mut self, val: u32) -> response::Result<()>;
  fn write_long(&mut self, val: i64) -> response::Result<()>;
  fn write_ulong(&mut self, val: u64) -> response::Result<()>;
  fn write_float(&mut self, val: f32) -> response::Result<()>;
  fn write_double(&mut self, val: f64) -> response::Result<()>;
  fn write_short_str(&mut self, val: String) -> response::Result<()>;
  fn write_long_str(&mut self, val: String) -> response::Result<()>;
  fn write_field_value_pair(&mut self, val: (String, ServerProperty)) -> response::Result<()>;
  fn write_field_value(&mut self, val: ServerProperty) -> response::Result<()>;
  fn write_argument(&mut self,  val: ServerProperty) -> response::Result<()>;
  fn write_prop_table(&mut self, val: HashMap<String, ServerProperty>) -> response::Result<()>;
}

impl <T: std::io::Write + ?Sized> Encode for T {
  fn write_bool(&mut self, val: bool) -> response::Result<()> {
    self.write_u8( if val { 1 } else { 0 })?;
    Ok(())
  }

  fn write_byte(&mut self, val: u8) -> response::Result<()> {
    self.write_u8(val)?;
    Ok(())
  }

  fn write_short(&mut self, val: i16) -> response::Result<()> {
    self.write_i16::<BigEndian>(val)?;
    Ok(())
  }

  fn write_ushort(&mut self, val: u16) -> response::Result<()> {
    self.write_u16::<BigEndian>(val)?;
    Ok(())
  }

  fn write_int(&mut self, val: i32) -> response::Result<()> {
    self.write_i32::<BigEndian>(val)?;
    Ok(())
  }

  fn write_uint(&mut self, val: u32) -> response::Result<()> {
    self.write_u32::<BigEndian>(val)?;
    Ok(())
  }

  fn write_long(&mut self, val: i64) -> response::Result<()> {
    self.write_i64::<BigEndian>(val)?;
    Ok(())
  }

  fn write_ulong(&mut self, val: u64) -> response::Result<()> {
    self.write_u64::<BigEndian>(val)?;
    Ok(())
  }

  fn write_float(&mut self, val: f32) -> response::Result<()> {
    self.write_f32::<BigEndian>(val)?;
    Ok(())
  }

  fn write_double(&mut self, val: f64) -> response::Result<()> {
    self.write_f64::<BigEndian>(val)?;
    Ok(())
  }

  fn write_short_str(&mut self, val: String) -> response::Result<()> {
    let str_bytes = val.into_bytes();
    // str_bytes.reverse();
    self.write_byte(str_bytes.len() as u8)?;
    self.write(& str_bytes)?;
    Ok(())
  }

  fn write_long_str(&mut self, val: String) -> response::Result<()> {
    let str_bytes = val.into_bytes();
    // str_bytes.reverse();
    Encode::write_uint(self, str_bytes.len() as u32)?;
    self.write(& str_bytes)?;
    Ok(())
  }

  fn write_field_value_pair(&mut self, val: (String, ServerProperty)) -> response::Result<()> {
    self.write_short_str(val.0)?;
    self.write_field_value(val.1)?;
    Ok(())
  }

  fn write_field_value(&mut self,  val: ServerProperty) -> response::Result<()> {
    match val {
      ServerProperty::Bool(v) => {
        self.write_byte('t' as u8)?;
        self.write_bool(v)?;
      },
      ServerProperty::Byte(v) => {
        self.write_byte('b' as u8)?;
        self.write_byte(v)?;
      },
      ServerProperty::Short(v) => {
        self.write_byte('U' as u8)?;
        self.write_short(v)?;
      },
      ServerProperty::UShort(v) => {
        self.write_byte('u' as u8)?;
        self.write_ushort(v)?;
      }
      ServerProperty::Int(v) => {
        self.write_byte('I' as u8)?;
        Encode::write_int(self, v)?;
      }
      ServerProperty::UInt(v) => {
        self.write_byte('i' as u8)?;
        Encode::write_uint(self, v)?;
      }
      ServerProperty::Long(v) => {
        self.write_byte('L' as u8)?;
        self.write_long(v)?;
      }
      ServerProperty::ULong(v) => {
        self.write_byte('l' as u8)?;
        self.write_ulong(v)?;
      }
      ServerProperty::Float(v) => {
        self.write_byte('f' as u8)?;
        self.write_float(v)?;
      }
      ServerProperty::Double(v) => {
        self.write_byte('d' as u8)?;
        self.write_double(v)?;
      }
      ServerProperty::ShortStr(v) => {
        self.write_byte('s' as u8)?;
        self.write_short_str(v)?;
      }
      ServerProperty::LongStr(v) => {
        self.write_byte('S' as u8)?;
        self.write_long_str(v)?;
      }
      ServerProperty::PropTable(v) => {
        self.write_byte('F' as u8)?;
        self.write_prop_table(v)?;
      }
    }

    Ok(())
  }

  fn write_argument(&mut self,  val: ServerProperty) -> response::Result<()> {
    match val {
      ServerProperty::Bool(v) => {
        self.write_bool(v)?;
      },
      ServerProperty::Byte(v) => {
        self.write_byte(v)?;
      },
      ServerProperty::Short(v) => {
        self.write_short(v)?;
      },
      ServerProperty::UShort(v) => {
        self.write_ushort(v)?;
      }
      ServerProperty::Int(v) => {
        Encode::write_int(self, v)?;
      }
      ServerProperty::UInt(v) => {
        Encode::write_uint(self, v)?;
      }
      ServerProperty::Long(v) => {
        self.write_long(v)?;
      }
      ServerProperty::ULong(v) => {
        self.write_ulong(v)?;
      }
      ServerProperty::Float(v) => {
        self.write_float(v)?;
      }
      ServerProperty::Double(v) => {
        self.write_double(v)?;
      }
      ServerProperty::ShortStr(v) => {
        self.write_short_str(v)?;
      }
      ServerProperty::LongStr(v) => {
        self.write_long_str(v)?;
      }
      ServerProperty::PropTable(v) => {
        self.write_prop_table(v)?;
      }
    }

    Ok(())
  }
  fn write_prop_table(&mut self, val: HashMap<String, ServerProperty>) -> response::Result<()> {
    let mut buff = vec![];

    for pair in val {
      buff.write_field_value_pair(pair)?;
    }

    Encode::write_uint(self, buff.len() as u32)?;
    info!("Write prop table size {} ", buff.len() as u32);
    self.write(& buff)?;
    Ok(())
  }
}
