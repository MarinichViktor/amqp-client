use crate::protocol::methods::channel::{OpenOk as ChanOpenOk};
use crate::protocol::methods::connection::{Start, Tune, OpenOk};

#[derive(Debug)]
pub struct AmqpFrame {
  pub ty: AmqpFrameType,
  pub chan: i16,
  pub method_payload: Option<Method>
}


#[derive(Debug)]
pub enum AmqpFrameType {
  Method,
  Heartbeat
}

impl AmqpFrame {
  pub fn method(chan: i16, payload: Method) -> Self {
    Self {
      ty: AmqpFrameType::Method,
      chan,
      method_payload: Some(payload)
    }
  }
}

#[deprecated]
#[derive(Debug)]
pub enum Frame {
  Method(MethodFrame)
}

#[derive(Debug)]
pub struct MethodFrame {
  pub chan: i16,
  pub payload: Method
}

// todo: refactor this mess
#[derive(Debug)]
pub enum Method {
  ConnStart(Start),
  ConnTune(Tune),
  ConnOpenOk(OpenOk),
  ChanOpenOk(ChanOpenOk),
  // ConnStartOk(StartOk),
}
