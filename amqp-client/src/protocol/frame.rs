#[derive(Debug)]
pub enum AmqFrame {
  Method(AmqMethodFrame),
  Header(AmqHeaderFrame),
  Body(AmqBodyFrame),
  Heartbeat
}

#[derive(Debug)]
pub struct AmqMethodFrame {
  pub chan: i16,
  pub class_id: i16,
  pub method_id: i16,
  pub body: Vec<u8>,
  pub content_header: Option<AmqHeaderFrame>,
  pub content_body: Option<Vec<u8>>,
}

impl AmqMethodFrame {
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
pub struct AmqHeaderFrame {
  pub chan: i16,
  pub class_id: i16,
  pub body_len: i64,
  pub prop_flags: i16,
  pub prop_list: Vec<u8>,
}

#[derive(Debug)]
pub struct AmqBodyFrame {
  pub chan: i16,
  pub body: Vec<u8>,
}
