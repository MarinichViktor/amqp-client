use crate::protocol::methods::connection::{Start, Tune};

// #[derive(Debug)]
// pub struct AmqpFrame {
//   ty: u8,
//   chan: i16,
//   size: i32,
//   body: Vec<u8>
// }

#[derive(Debug)]
pub enum Frame {
  Method(MethodFrame)
}

#[derive(Debug)]
pub struct MethodFrame {
  pub chan: i16,
  pub payload: Method
}

#[derive(Debug)]
pub enum Method {
  ConnStart(Start),
  ConnTune(Tune),
  // ConnStartOk(StartOk),
}
