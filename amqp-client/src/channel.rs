use std::sync::Arc;
use crate::protocol::channel::AmqChannel;
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

  pub fn declare_exchange(&self) -> Result<String> {
  }
}
