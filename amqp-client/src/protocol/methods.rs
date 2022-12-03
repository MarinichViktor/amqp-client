use std::collections::HashMap;
use amqp_protocol::types::{Property};

pub mod connection;
pub mod channel;

pub type PropTable = HashMap<String, Property>;
