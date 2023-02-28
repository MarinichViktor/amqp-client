// extern crate core;
pub(crate) mod protocol;
pub(crate) mod utils;
pub(crate) mod connection;
pub(crate) mod channel;
pub use amqp_protocol::response::{Result,Error,bail};
pub use crate::protocol::{
  connection::AmqConnection as Connection,
  channel::AmqChannel as Channel
};
pub use crate::protocol::exchange::ExchangeType as ExchangeType;
