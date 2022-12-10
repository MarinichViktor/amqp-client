use std::sync::Arc;
use crate::protocol::channel::AmqChannel;
use crate::protocol::exchange::{ExchangeDeclareOptsBuilder, ExchangeType};
use crate::Result;

pub struct Channel {
  raw: Arc<AmqChannel>
}

impl Channel {
  pub(crate) fn new(raw: Arc<AmqChannel>) -> Self {
    Channel {
      raw
    }
  }

  pub fn open(&self) -> Result<()> {
    self.raw.open()
  }

  pub fn close(&self) -> Result<()> {
    self.raw.close()
  }

  pub fn declare_exchange<F>(&self, configure: F) -> Result<String>
    where F: FnMut(&mut ExchangeDeclareOptsBuilder)
  {
    self.raw.declare_exchange(configure)
  }
}
