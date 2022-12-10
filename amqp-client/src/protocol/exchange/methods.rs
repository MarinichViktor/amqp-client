use amqp_macros::amqp_method;
use amqp_protocol::types::Table;

#[derive(Debug, Default)]
#[amqp_method(c_id=30, m_id=10)]
pub struct Declare {
  #[short_str]
  pub reserved1: String,
  #[short_str]
  pub exchange: String,
  #[short_str]
  pub ty: String,
  #[byte]
  pub passive: u8,
  #[byte]
  pub durable: u8,
  #[byte]
  pub auto_delete: u8,
  #[byte]
  pub internal: u8,
  #[byte]
  pub no_wait: u8,
  #[prop_table]
  pub table: Table,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=30, m_id=11)]
pub struct DeclareOk {
}

#[derive(Debug, Default)]
#[amqp_method(c_id=30, m_id=20)]
pub struct Delete {
  #[short_str]
  pub reserved1: String,
  #[short_str]
  pub exchange: String,
  #[byte]
  pub if_unused: u8,
  #[byte]
  pub no_wait: u8,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=30, m_id=21)]
pub struct DeleteOk {
}
