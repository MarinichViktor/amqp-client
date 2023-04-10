use std::collections::HashMap;
use crate::protocol::types::{PropTable, QueueDeclare, ShortStr};

pub struct QueueDeclareOpts {
  pub name: String,
  pub passive: bool,
  pub durable: bool,
  pub exclusive: bool,
  pub auto_delete: bool,
  pub no_wait: bool,
  pub props: PropTable
}

impl Default for QueueDeclareOpts {
  fn default() -> Self {
    Self {
      // todo: add name generator
      name: "".to_string(),
      passive: false,
      durable: true,
      exclusive: false,
      auto_delete: false,
      no_wait: false,
      props: HashMap::new()
    }
  }
}

pub struct QueueDeclareOptsBuilder {
  opts: QueueDeclareOpts
}

impl QueueDeclareOptsBuilder {
  pub fn new() -> Self {
    Self {
      opts: QueueDeclareOpts::default()
    }
  }

  pub fn build(self) -> QueueDeclareOpts {
    self.opts
  }

  pub fn name(&mut self, name: String) {
    self.opts.name = name;
  }

  pub fn passive(&mut self, passive: bool) {
    self.opts.passive = passive;
  }

  pub fn durable(&mut self, durable: bool) {
    self.opts.durable = durable;
  }

  pub fn exclusive(&mut self, exclusive: bool) {
    self.opts.exclusive = exclusive;
  }

  pub fn auto_delete(&mut self, auto_delete: bool) {
    self.opts.auto_delete = auto_delete;
  }

  pub fn no_wait(&mut self, no_wait: bool) {
    self.opts.no_wait = no_wait;
  }

  pub fn props(&mut self, props: PropTable) {
    self.opts.props = props;
  }
}
const PASSIVE_MASK: u8 = 0b01;
const DURABLE_MASK: u8 = 0b10;
const EXCLUSIVE_MASK: u8 = 0b100;
const AUTODELETE_MASK: u8 = 0b1000;
const NOWAIT_MASK: u8 = 0b10000;

impl From<QueueDeclareOpts> for QueueDeclare {
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
      name: options.name.into(),
      flags,
      props: options.props
    }
  }
}
