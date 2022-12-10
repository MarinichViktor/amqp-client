use std::collections::HashMap;
use amqp_protocol::types::Table;

pub mod methods;
pub mod constants;

pub enum ExchangeType {
  Direct,
  Fanout,
  // todo: exhaustive list
}

pub struct ExchangeDeclareOpts {
  pub name: String,
  pub ty: ExchangeType,
  pub passive: bool,
  pub durable: bool,
  pub auto_delete: bool,
  pub internal: bool,
  pub no_wait: bool,
  pub props: Table
}

impl Default for ExchangeDeclareOpts {
  fn default() -> Self {
    Self {
      // todo: add name generator
      name: "".to_string(),
      ty: ExchangeType::Direct,
      passive: false,
      durable: true,
      auto_delete: false,
      internal: false,
      no_wait: false,
      props: HashMap::new()
    }
  }
}

pub struct ExchangeDeclareOptsBuilder {
  opts: ExchangeDeclareOpts
}

impl ExchangeDeclareOptsBuilder {
  pub fn new() -> Self {
    Self {
      opts: ExchangeDeclareOpts::default()
    }
  }

  pub fn build(self) -> ExchangeDeclareOpts {
    self.opts
  }

  pub fn name(&mut self, name: String) {
    self.opts.name = name;
  }

  pub fn ty(&mut self, ty: ExchangeType) {
    self.opts.ty = ty;
  }

  pub fn passive(&mut self, passive: bool) {
    self.opts.passive = passive;
  }

  pub fn durable(&mut self, durable: bool) {
    self.opts.durable = durable;
  }

  pub fn auto_delete(&mut self, auto_delete: bool) {
    self.opts.auto_delete = auto_delete;
  }

  pub fn internal(&mut self, internal: bool) {
    self.opts.internal = internal;
  }

  pub fn no_wait(&mut self, no_wait: bool) {
    self.opts.no_wait = no_wait;
  }

  pub fn props(&mut self, props: Table) {
    self.opts.props = props;
  }
}
