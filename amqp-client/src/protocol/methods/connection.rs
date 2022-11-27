use amqp_macros::amqp_method;
use crate::connection::DefaultChan;
use crate::protocol::methods::PropTable;

pub const CLASS_CONNECTION: i16 = 10;
pub const METHOD_START: i16 = 10;
pub const METHOD_STARTOK: i16 = 11;
pub const METHOD_SECURE: i16 = 20;
pub const METHOD_SECUREOK: i16 = 21;
pub const METHOD_TUNE: i16 = 30;
pub const METHOD_TUNEOK: i16 = 31;
pub const METHOD_OPEN: i16 = 40;
pub const METHOD_OPENOK: i16 = 41;

#[derive(Debug)]
#[amqp_method(c_id=10, m_id=10)]
pub struct Start {
  #[byte]
  pub ver_major: u8,
  #[byte]
  pub ver_minor: u8,
  #[prop_table]
  pub properties: PropTable,
  #[long_str]
  pub mechanisms: String,
  #[long_str]
  pub locales: String,
}

#[derive(Debug)]
#[amqp_method(c_id=10, m_id=11)]
pub struct StartOk {
  #[prop_table]
  pub properties: PropTable,
  #[short_str]
  pub mechanism: String,
  #[long_str]
  pub response: String,
  #[short_str]
  pub locale: String,
}

#[derive(Debug)]
#[amqp_method(c_id=10, m_id=30)]
pub struct Tune {
  #[short]
  pub chan_max: i16,
  #[int]
  pub frame_max: i32,
  #[short]
  pub heartbeat: i16,
}

#[derive(Debug)]
#[amqp_method(c_id=10, m_id=31)]
pub struct TuneOk {
  #[short]
  pub chan_max: i16,
  #[int]
  pub frame_max: i32,
  #[short]
  pub heartbeat: i16,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=10, m_id=40)]
pub struct Open {
  #[short_str]
  pub vhost: String,
  #[short_str]
  pub reserved1: String,
  #[byte]
  pub reserved2: u8,
}


#[derive(Debug, Default)]
#[amqp_method(c_id=10, m_id=41)]
pub struct OpenOk {
  #[short_str]
  pub reserved1: String,
}
