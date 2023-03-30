use amqp_macros::amqp_method;
use amqp_protocol::types::Table;
use crate::queue::QueueDeclareOpts;

const PASSIVE_MASK: u8 = 0b01;
const DURABLE_MASK: u8 = 0b10;
const EXCLUSIVE_MASK: u8 = 0b100;
const AUTODELETE_MASK: u8 = 0b1000;
const NOWAIT_MASK: u8 = 0b10000;

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=10)]
pub struct Declare {
  #[short]
  pub reserved1: i16,
  #[short_str]
  pub name: String,
  #[byte]
  pub flags: u8,
  #[prop_table]
  pub table: Table,
}

impl From<QueueDeclareOpts> for Declare {
  fn from(options: QueueDeclareOpts) -> Self {
    let mut flags = 0;

    if options.passive {
      flags = flags & PASSIVE_MASK;
    }

    if options.durable {
      flags = flags & DURABLE_MASK;
    }

    if options.exclusive {
      flags = flags & EXCLUSIVE_MASK;
    }

    if options.auto_delete {
      flags = flags & AUTODELETE_MASK;
    }

    if options.no_wait {
      flags = flags & NOWAIT_MASK;
    }

    Self {
      reserved1: 0,
      name: options.name,
      flags,
      table: options.props
    }
  }
}

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=11)]
pub struct DeclareOk {
  #[short_str]
  pub name: String,
  #[int]
  pub message_count: i32,
  #[int]
  pub consumer_count: i32,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=20)]
pub struct Bind {
  #[short]
  pub reserved1: i16,
  #[short_str]
  pub queue_name: String,
  #[short_str]
  pub exchange_name: String,
  #[short_str]
  pub routing_key: String,
  #[byte]
  pub no_wait: u8,
  #[prop_table]
  pub table: Table,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=21)]
pub struct BindOk {}

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=50)]
pub struct Unbind {
  #[short]
  pub reserved1: i16,
  #[short_str]
  pub queue_name: String,
  #[short_str]
  pub exchange_name: String,
  #[short_str]
  pub routing_key: String,
  #[prop_table]
  pub table: Table,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=50, m_id=51)]
pub struct UnbindOk {}
