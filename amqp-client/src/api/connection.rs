use std::collections::HashMap;
use std::io::Write;
use bytes::buf::Writer;

use log::{info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::{mpsc};

use crate::protocol::types::{ConnectionOpen, ConnectionStartOk, ConnectionTuneOk, LongStr, Property, ShortStr};

use crate::{Result};
use crate::api::connection::options::ConnectionArgs;
use crate::api::connection::constants::PROTOCOL_HEADER;
use crate::default_channel::{DEFAULT_CHANNEL_ID, DefaultChannel};
use crate::protocol::PropTable;
use self::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;
use crate::utils::IdAllocator;
use self::options::ConnectionAddress;
use crate::protocol::types::Frame;

pub mod constants;
pub mod methods;
pub mod factory;
pub mod options;
pub use self::factory::ConnectionFactory;

pub struct Connection {
  arguments: ConnectionArgs,
  id_allocator: IdAllocator,
}

macro_rules! unwrap_frame_variant {
  (
    $enum: expr, $variant:ident
  ) => {
    match $enum {
      Frame::$variant(payload) => payload,
      _ => panic!("Failed to unwrap variant")
    }

  }
}

impl Connection {
  pub async fn open(stream: TcpStream, args: ConnectionArgs) -> Result<Connection> {
    let stream_parts = stream.into_split();
    let mut reader = FrameReader::new(BufReader::new(stream_parts.0));
    let mut writer = FrameWriter::new(BufWriter::new(stream_parts.1));

    let connection = Self {
      arguments: args,
      id_allocator: IdAllocator::new(),
    };

    connection.handshake(&mut reader, &mut writer).await?;

    Ok(connection)
  }

  async fn handshake(&self, reader: &mut FrameReader, writer: &mut FrameWriter) -> Result<()> {
    info!("handshake started");
    writer.write_binary(&PROTOCOL_HEADER).await?;

    let frame = reader.next_frame().await?;
    let _start_method = unwrap_frame_variant!(frame, ConnectionStart);

    let client_properties: PropTable = HashMap::from([
      (ShortStr("product".to_string()), Property::LongStr(LongStr(PRODUCT.to_string()))),
      (ShortStr("platform".to_string()), Property::LongStr(LongStr(PLATFORM.to_string()))),
      (ShortStr("copyright".to_string()), Property::LongStr(LongStr(COPYRIGHT.to_string()))),
      (ShortStr("information".to_string()), Property::LongStr(LongStr(INFORMATION.to_string())))
    ]);
    let start_ok_method = ConnectionStartOk {
      properties: client_properties,
      mechanism: ShortStr(DEFAULT_AUTH_MECHANISM.to_string()),
      response: LongStr(format!("\x00{}\x00{}", self.arguments.address.login.as_str(), self.arguments.address.password)),
      locale: ShortStr(DEFAULT_LOCALE.to_string()),
    };

    writer.send_frame(0, start_ok_method.into_frame()).await?;
    let frame = reader.next_frame().await?;
    let _tune_method = unwrap_frame_variant!(frame, ConnectionTune);

    let tune_ok_method = ConnectionTuneOk {
      chan_max: self.arguments.max_channels,
      frame_max: self.arguments.max_frame_size,
      heartbeat: self.arguments.heartbeat_interval
    };

    writer.send_frame(0, tune_ok_method.into_frame()).await?;

    let open_method = ConnectionOpen {
      vhost: self.arguments.address.vhost.clone().into(),
      reserved1: "".into(),
      reserved2: 0
    };

    writer.send_frame(0, open_method.into_frame()).await?;

    let frame = reader.next_frame().await?;
    let _open_ok_method = unwrap_frame_variant!(frame, ConnectionOpenOk);
    info!("handshake completed");

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
