use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};

use amqp_protocol::types::Property;

use crate::{Result};
use crate::protocol::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::connection::options::ConnectionOpts;
use crate::protocol::frame::{Frame, HeaderFrame, MethodFrame};
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
