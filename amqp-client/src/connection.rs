use std::collections::HashMap;
use std::sync::{Arc};

use log::{info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex, oneshot};

use amqp_protocol::types::Property;

use crate::{Channel, Result};
use crate::protocol::amqp_connection::AmqpConnection;
use crate::protocol::basic;
use crate::protocol::basic::methods::Deliver;
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
  internal: AmqpConnection,
  options: ConnectionOpts,
  id_allocator: IdAllocator,
  channels: Arc<Mutex<HashMap<i16, mpsc::Sender<RawFrame>>>>,
  frame_tx: mpsc::Sender<RawFrame>,
  frame_rx: Option<mpsc::Receiver<RawFrame>>,
  sync_waiter_queue: Arc<Mutex<HashMap<i16, Vec<oneshot::Sender<RawFrame>>>>>
}

impl Connection {
  pub fn new(stream: TcpStream, options: ConnectionOpts) -> Self {
    let stream_parts = stream.into_split();
    let reader = FrameReader::new(BufReader::new(stream_parts.0));
    let writer = FrameWriter::new(BufWriter::new(stream_parts.1));
    let (sender, receiver) = mpsc::channel(128);

    Self {
      internal: AmqpConnection::new(reader, writer),
      options,
      id_allocator: IdAllocator::new(),
      channels: Arc::new(Mutex::new(HashMap::new())),
      frame_tx: sender,
      frame_rx: Some(receiver),
      sync_waiter_queue: Default::default()
    }
  }

  pub async fn connect(&mut self) -> Result<()> {
    self.handshake().await?;
    self.start_listener();
    Ok(())
  }

  async fn handshake(&mut self) -> Result<()> {
    use self::constants::PROTOCOL_HEADER;
    use self::methods as conn_methods;

    info!("Handshake started");
    let mut writer = self.internal.writer.lock().await;
    let reader = self.internal.reader.as_mut().unwrap();

    info!("Sending [ProtocolHeader]");
    writer.write_all(&PROTOCOL_HEADER).await?;
    let _start_method: Frame2<conn_methods::Start> = reader.next_frame().await?.into();

    info!("Sending [StartOk]");
    let client_properties = HashMap::from([
      ("product".to_string(), Property::LongStr(PRODUCT.to_string())),
      ("platform".to_string(), Property::LongStr(PLATFORM.to_string())),
      ("copyright".to_string(), Property::LongStr(COPYRIGHT.to_string())),
      ("information".to_string(), Property::LongStr(INFORMATION.to_string()))
    ]);
    // todo: add const for default channel or separate struct
    let start_ok = conn_methods::StartOk {
      properties: client_properties,
      mechanism: DEFAULT_AUTH_MECHANISM.to_string(),
      response: format!("\x00{}\x00{}", self.options.login.as_str(), self.options.password),
      locale: DEFAULT_LOCALE.to_string(),
    };
    writer.write3(0, start_ok).await?;

    let tune_method: Frame2<conn_methods::Tune> = reader.next_frame().await?.into();

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.args.chan_max,
      frame_max: tune_method.args.frame_max,
      heartbeat: tune_method.args.heartbeat,
    };
    info!("Sending [TuneOk]");

    writer.write3(0, tune_ok_method).await?;


    info!("Sending [OpenMethod]");
    let open_method_frame = conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    };
    writer.write3(0, open_method_frame).await?;
    info!("Handshake completed");

    let _open_ok_method: Frame2<conn_methods::OpenOk> = reader.next_frame().await?.into();
    Ok(())
  }

  pub fn start_listener(&mut self) {
    let mut reader = self.internal.reader.take().unwrap();
    let channels = self.channels.clone();
    let mut receiver = self.frame_rx.take().unwrap();
    let writer = self.internal.writer.clone();
    let handle = Handle::current();
    let sync_waiter_queue = self.sync_waiter_queue.clone();

    std::thread::spawn(move || {
      handle.spawn(async move {
        while let Ok(frame) = reader.next_frame().await {
          let channels_map = channels.lock().await;
          channels_map[&frame.ch].send(frame).await.unwrap();
        }
      });

      handle.spawn(async move {
        while let Some(request) = receiver.recv().await {
          let mut writer = writer.lock().await;
          writer.write(request).await.unwrap();
        }
      });
    });
  }

  pub async fn create_channel(&mut self) -> Result<Channel> {
    let id = self.id_allocator.allocate();

    info!("[Connection] create_channel {}", id);
    let mut channel = Channel::new(id, self.frame_tx.clone());
    {
      let mut channels = self.channels.lock().await;
      channels.insert(channel.id, channel.global_frame_transmitter.clone());
    }
    channel.open().await?;

    Ok(channel)
  }
}
