use std::collections::HashMap;
use amqp_protocol::types::Table;

pub mod constants;
pub mod methods;

pub struct QueueDeclareOpts {
  pub name: String,
  pub passive: bool,
  pub durable: bool,
  pub exclusive: bool,
  pub auto_delete: bool,
  pub no_wait: bool,
  pub props: Table
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

  pub fn props(&mut self, props: Table) {
    self.opts.props = props;
  }
}
