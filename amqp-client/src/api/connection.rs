use std::collections::HashMap;
use std::sync::Arc;

use log::{info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::protocol::types::{AmqpMessage, ConnectionOpen, ConnectionStartOk, ConnectionTuneOk, LongStr, Property, ShortStr};

use crate::{invoke_command_async, Result, unwrap_frame_variant};
use crate::api::channel::AmqChannel;
use crate::api::connection::options::ConnectionArgs;
use crate::api::connection::constants::PROTOCOL_HEADER;
use crate::internal::channel::{Command, CommandPayload, ChannelManager};
use crate::protocol::PropTable;
use self::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;
use crate::utils::IdAllocator;
use crate::protocol::types::Frame;

pub mod constants;
pub mod factory;
pub mod options;
pub use self::factory::ConnectionFactory;


pub struct Connection {
  arguments: ConnectionArgs,
  id_allocator: IdAllocator,
  message_tx: UnboundedSender<AmqpMessage>,
  command_tx: UnboundedSender<Command>,
  channel_manager: Arc<Mutex<ChannelManager>>
}

impl Connection {
  pub async fn open(stream: TcpStream, args: ConnectionArgs) -> Result<Connection> {
    let stream_parts = stream.into_split();
    let mut reader = FrameReader::new(BufReader::new(stream_parts.0));
    let mut writer = FrameWriter::new(BufWriter::new(stream_parts.1));

    let (msg_tx, msg_rx) = mpsc::unbounded_channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();

    let connection = Self {
      arguments: args,
      id_allocator: IdAllocator::new(),
      message_tx: msg_tx,
      command_tx,
      channel_manager: Arc::new(Mutex::new(ChannelManager::new())),
    };

    connection.handshake(&mut reader, &mut writer).await?;
    connection.spawn_connection_handlers(reader, writer, msg_rx, command_rx);

    Ok(connection)
  }

  fn spawn_connection_handlers(&self, mut reader: FrameReader, mut writer: FrameWriter, mut msg_rx: UnboundedReceiver<AmqpMessage>, mut cmd_rx: UnboundedReceiver<Command>) {
    let mut channel_manager = ChannelManager::new();

    tokio::spawn(async move {
      loop {
        tokio::select! {
          command = cmd_rx.recv() => {
            let (payload, acker) = command.unwrap();
            match payload {
              CommandPayload::RegisterResponder((channel, responder)) => {
                channel_manager.register_responder(channel, responder);
                acker.send(()).unwrap();
              },
              CommandPayload::RegisterChannel((id, incoming_tx)) => {
                channel_manager.register_channel(id, incoming_tx);
                acker.send(()).unwrap();
              },
              CommandPayload::RegisterConsumer => {
              }
            }
          },
          frame = reader.next_frame() => {
            let (channel, frame) = frame.unwrap();
            match &frame {
              Frame::Heartbeat => {
                todo!("Do something with heartbeat");
              },
              Frame::ContentHeader | Frame::ContentBody => {
                todo!("Add content handlers");
              },
              Frame::ChannelOpenOk(..) |
              Frame::ExchangeDeclareOk(..) |
              Frame::QueueDeclareOk(..) |
              Frame::QueueBindOk(..) |
              Frame::QueueUnbindOk(..) => {
                channel_manager.get_responder(channel).send(frame).unwrap();
              }
              frame => {
                println!("frame received {:?}", frame);
              }
            }
          }
        }
      }
    });

    tokio::spawn(async move {
      while let Some((channel, frame)) = msg_rx.recv().await {
        writer.dispatch(channel, frame).await.unwrap();
      }
    });
  }

  async fn handshake(&self, reader: &mut FrameReader, writer: &mut FrameWriter) -> Result<()> {
    info!("handshake started");
    writer.write_binary(&PROTOCOL_HEADER).await?;

    let (_, frame) = reader.next_frame().await?;
    let _start_method = unwrap_frame_variant!(frame, ConnectionStart);

    let client_properties: PropTable = HashMap::from([
      ("product".into(), Property::LongStr(PRODUCT.into())),
      ("platform".into(), Property::LongStr(PLATFORM.into())),
      ("copyright".into(), Property::LongStr(COPYRIGHT.into())),
      ("information".into(), Property::LongStr(INFORMATION.into()))
    ]);
    let start_ok_method = ConnectionStartOk {
      properties: client_properties,
      mechanism: ShortStr(DEFAULT_AUTH_MECHANISM.to_string()),
      response: LongStr(format!("\x00{}\x00{}", self.arguments.address.login.as_str(), self.arguments.address.password)),
      locale: ShortStr(DEFAULT_LOCALE.to_string()),
    };

    writer.dispatch(0, start_ok_method.into_frame()).await?;
    let (_, frame) = reader.next_frame().await?;
    let _tune_method = unwrap_frame_variant!(frame, ConnectionTune);

    let tune_ok_method = ConnectionTuneOk {
      chan_max: self.arguments.max_channels,
      frame_max: self.arguments.max_frame_size,
      heartbeat: self.arguments.heartbeat_interval
    };

    writer.dispatch(0, tune_ok_method.into_frame()).await?;

    let open_method = ConnectionOpen {
      vhost: self.arguments.address.vhost.clone().into(),
      reserved1: "".into(),
      reserved2: 0
    };

    writer.dispatch(0, open_method.into_frame()).await?;

    let (_, frame) = reader.next_frame().await?;
    let _open_ok_method = unwrap_frame_variant!(frame, ConnectionOpenOk);
    info!("handshake completed");

    Ok(())
  }

  pub async fn create_channel(&mut self) -> Result<AmqChannel> {
    let id = self.id_allocator.allocate();
    info!("create channel");

    let (channel_tx, channel_rx) = mpsc::unbounded_channel();

    invoke_command_async!(self.command_tx, CommandPayload::RegisterChannel((id, channel_tx)));

    let channel = AmqChannel::open(id, self.message_tx.clone(), channel_rx, self.command_tx.clone()).await?;

    info!("channel created");
    Ok(channel)
  }
}
