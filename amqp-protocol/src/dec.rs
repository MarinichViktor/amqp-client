use std::collections::HashMap;
use std::io::{Cursor};
use byteorder::{BigEndian, ReadBytesExt};
use log::{debug};
use crate::types::Property;
use crate::response;

pub trait Decode {
  fn read_bool(&mut self) -> response::Result<bool>;
  fn read_byte(&mut self) -> response::Result<u8>;
  fn read_short(&mut self) -> response::Result<i16>;
  fn read_ushort(&mut self) -> response::Result<u16>;
  fn read_int(&mut self) -> response::Result<i32>;
  fn read_uint(&mut self) -> response::Result<u32>;
  fn read_long(&mut self) -> response::Result<i64>;
  fn read_ulong(&mut self) -> response::Result<u64>;
  fn read_float(&mut self) -> response::Result<f32>;
  fn read_double(&mut self) -> response::Result<f64>;
  fn read_short_str(&mut self) -> response::Result<String>;
  fn read_long_str(&mut self) -> response::Result<String>;
  fn read_field_value_pair(&mut self) -> response::Result<(String, Property)>;
  fn read_field_value(&mut self) -> response::Result<Property>;
  fn read_field_value_type(&mut self, ch: char) -> response::Result<Property>;
  fn read_prop_table(&mut self) -> response::Result<HashMap<String, Property>>;
}

impl <T: std::io::Read + ?Sized> Decode for T {
  fn read_bool(&mut self) -> response::Result<bool> {
    Ok(self.read_u8()? == 0)
  }
  fn read_byte(&mut self) -> response::Result<u8> {
    Ok(self.read_u8()?)
  }

  fn read_short(&mut self) -> response::Result<i16> {
    Ok(self.read_i16::<BigEndian>()?)
  }

  fn read_ushort(&mut self) -> response::Result<u16> {
    Ok(self.read_u16::<BigEndian>()?)
  }

  fn read_int(&mut self) -> response::Result<i32> {
    Ok(self.read_i32::<BigEndian>()?)
  }

  fn read_uint(&mut self) -> response::Result<u32> {
    Ok(self.read_u32::<BigEndian>()?)
  }

  fn read_long(&mut self) -> response::Result<i64> {
    Ok(self.read_i64::<BigEndian>()?)
  }

  fn read_ulong(&mut self) -> response::Result<u64> {
    Ok(self.read_u64::<BigEndian>()?)
  }

  fn read_float(&mut self) -> response::Result<f32> {
    Ok(self.read_f32::<BigEndian>()?)
  }

  fn read_double(&mut self) -> response::Result<f64> {
    Ok(self.read_f64::<BigEndian>()?)
  }

  fn read_short_str(&mut self) -> response::Result<String> {
    let size = self.read_byte()?;
    let mut buff = vec![0_u8; size as usize];
    self.read_exact(&mut buff)?;
    Ok(String::from_utf8(buff)?)
  }

  fn read_long_str(&mut self) -> response::Result<String> {
    let size = Decode::read_uint(self)?;
    let mut buff = vec![0_u8; size as usize];
    self.read_exact(&mut buff)?;
    Ok(String::from_utf8(buff)?)
  }

  fn read_field_value_pair(&mut self) -> response::Result<(String, Property)> {
    let key = self.read_short_str()?;
    let value = self.read_field_value()?;
    Ok((key, value))
  }

  fn read_field_value(&mut self) -> response::Result<Property> {
    let value_type = self.read_byte()? as char;
    Ok(self.read_field_value_type(value_type)?)
  }

  fn read_field_value_type(&mut self, ch: char) -> response::Result<Property> {
    let value = match ch {
      't' => Property::Bool(self.read_bool()?),
      'b' | 'B' => Property::Byte(self.read_byte()?),
      'U' => Property::Short(self.read_short()?),
      'u' => Property::UShort(self.read_ushort()?),
      'I' => Property::Int(Decode::read_int(self)?),
      'i' => Property::UInt(Decode::read_uint(self)?),
      'L' => Property::Long(self.read_long()?),
      'l' => Property::ULong(self.read_ulong()?),
      'f' => Property::Float(self.read_float()?),
      'd' => Property::Double(self.read_double()?),
      's' => Property::ShortStr(self.read_short_str()?),
      'S' => Property::LongStr(self.read_long_str()?),
      'F' => Property::Table(self.read_prop_table()?),
      _ => {
        panic!("Unexpected values provided: {}", ch);
      }
    };

    Ok(value)
  }

  fn read_prop_table(&mut self) -> response::Result<HashMap<String, Property>> {
    let mut table = HashMap::new();
    let table_size = Decode::read_uint(self)?;
    debug!("Table size {}", table_size);
    let mut buff = vec![0_u8; table_size as usize];
    self.read_exact(& mut buff)?;
    let mut cursor = Cursor::new(buff);

    while cursor.position() < table_size as u64 - 1 {
      let pair = cursor.read_field_value_pair()?;
      debug!("Table pair {:?}", &pair);
      table.insert(pair.0, pair.1);
    }

    Ok(table)
  }
}
