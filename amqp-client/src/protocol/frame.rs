#[derive(Debug)]
pub enum AmqFrame {
  Method(AmqMethodFrame),
  Heartbeat
}

#[derive(Debug)]
pub struct AmqMethodFrame {
  pub chan: i16,
  pub class_id: i16,
  pub method_id: i16,
  pub body: Vec<u8>
}
