use std::sync::Arc;

use tokio::sync::{Mutex};

use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;

pub struct AmqpConnection {
  pub reader: Option<FrameReader>,
  pub writer: Arc<Mutex<FrameWriter>>,
}

impl AmqpConnection {
  pub fn new(reader: FrameReader, writer: FrameWriter) -> Self {
    Self {
      reader: Some(reader),
      writer: Arc::new(Mutex::new(writer))
    }
  }
}
