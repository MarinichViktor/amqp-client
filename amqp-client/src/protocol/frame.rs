#[derive(Debug)]
pub enum Frame {
  Method(MethodFrame),
  Header(HeaderFrame),
  Body(BodyFrame),
  Heartbeat
}

#[derive(Debug)]
pub struct MethodFrame {
  pub chan: i16,
  pub class_id: i16,
  pub method_id: i16,
  pub body: Vec<u8>,
}

impl MethodFrame {
  pub fn has_content(&self) -> bool {
    // todo: currently hardcoded only to check if deliver method
    if self.class_id == 60 && self.method_id == 60 {
      true
    } else {
      false
    }
  }
}

#[derive(Debug)]
pub struct HeaderFrame {
  pub chan: i16,
  pub class_id: i16,
  pub body_len: i64,
  pub prop_flags: i16,
  pub prop_list: Vec<u8>,
}

#[derive(Debug)]
pub struct BodyFrame {
  pub chan: i16,
  pub body: Vec<u8>,
}
