use crate::protocol::methods::channel::{OpenOk as ChanOpenOk};
use crate::protocol::methods::connection::{Start, Tune, OpenOk};

#[derive(Debug)]
pub struct AmqpFrame {
  ty: AmqpFrameType,
  chan: i16,
  method_payload: Option<Method>
}

#[derive(Debug)]
pub enum AmqpFrameType {
  Method,
  Heartbeat
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
