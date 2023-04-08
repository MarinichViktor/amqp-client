use std::collections::HashMap;

use log::{info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::{mpsc};

use crate::protocol::types::Property;

use crate::{Result};
use crate::default_channel::{DEFAULT_CHANNEL_ID, DefaultChannel};
use crate::protocol::amqp_connection::AmqpConnection;
use self::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;
use crate::utils::IdAllocator;
use self::options::ConnectionOpts;
use crate::protocol::frame2::{Frame2, RawFrame};

pub mod constants;
pub mod methods;
pub mod factory;
pub mod options;

pub type FrameTransmitter = mpsc::Sender<RawFrame>;

pub struct Connection {
  amqp_handle: AmqpConnection,
  options: ConnectionOpts,
  id_allocator: IdAllocator,
}

impl Connection {
  pub fn new(stream: TcpStream, options: ConnectionOpts) -> Self {
    let stream_parts = stream.into_split();
    let reader = FrameReader::new(BufReader::new(stream_parts.0));
    let writer = FrameWriter::new(BufWriter::new(stream_parts.1));

    Self {
      amqp_handle: AmqpConnection::new(reader, writer),
      options,
      id_allocator: IdAllocator::new(),
    }
  }

  pub async fn connect(&mut self) -> Result<()> {
    self.amqp_handle.start_frames_listener();

    let reader = self.amqp_handle.subscribe(DEFAULT_CHANNEL_ID).await;
    let mut default_channel = DefaultChannel::new(self.amqp_handle.get_writer().clone(), reader);
    default_channel.open(self.options.clone()).await;
    Ok(())
  }

  // pub async fn create_channel(&mut self) -> Result<Channel> {
  //   let id = self.id_allocator.allocate();
  //
  //   let rx = self.amqp_handle.subscribe(id).await;
  //
  //   let mut channel = Channel::new(id, self.amqp_handle.get_writer().clone(), rx);
  //   channel.open().await?;
  //
  //   Ok(channel)
  // }
}
