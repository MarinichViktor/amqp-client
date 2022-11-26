use amqp_macros::amqp_method;
use crate::protocol::methods::PropTable;

#[derive(Debug)]
pub enum ConnMethodArgs {
  Start(StartMethodArgs),
  StartOk(StartOkMethodArgs),
}

#[derive(Debug)]
#[amqp_method(c_id=10, m_id=10)]
pub struct StartMethodArgs {
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
pub struct StartOkMethodArgs {
  #[short]
  pub channel: i16,
  #[prop_table]
  pub properties: PropTable,
  #[short_str]
  pub mechanism: String,
  #[long_str]
  pub response: String,
  #[short_str]
  pub locale: String,
}
