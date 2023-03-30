use std::collections::HashMap;
use std::sync::{Arc};

use log::{info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};

use amqp_protocol::types::Property;

use crate::{Channel, Result};
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
  internal: AmqpConnection,
  options: ConnectionOpts,
  id_allocator: IdAllocator,
  channels: Arc<Mutex<HashMap<i16, mpsc::Sender<RawFrame>>>>,
  frame_tx: mpsc::Sender<RawFrame>,
  frame_rx: Option<mpsc::Receiver<RawFrame>>,
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
      frame_rx: Some(receiver)
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
    let start_ok = Frame2::new(0, conn_methods::StartOk {
      properties: client_properties,
      mechanism: DEFAULT_AUTH_MECHANISM.to_string(),
      response: format!("\x00{}\x00{}", self.options.login.as_str(), self.options.password),
      locale: DEFAULT_LOCALE.to_string(),
    });
    writer.write2(start_ok).await?;

    let tune_method: Frame2<conn_methods::Tune> = reader.next_frame().await?.into();

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.args.chan_max,
      frame_max: tune_method.args.frame_max,
      heartbeat: tune_method.args.heartbeat,
    };
    info!("Sending [TuneOk]");

    let tune_ok = Frame2::new(0, tune_ok_method);
    writer.write2(tune_ok).await?;


    info!("Sending [OpenMethod]");
    let open_method_frame = Frame2::new(0, conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    });
    writer.write2(open_method_frame).await?;
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

// const FRAME_HEADER_SIZE: usize = 7;
// const FRAME_END_SIZE: usize = 1;
//
// pub struct FrameReader {
//   inner: BufReader<OwnedReadHalf>,
//   buf: BytesMut,
// }
//
// impl FrameReader {
//   pub fn new(inner: BufReader<OwnedReadHalf>) -> Self {
//     Self {
//       inner,
//       // todo: review default capacity
//       buf: BytesMut::with_capacity(128 * 1024)
//     }
//   }
//
//   pub async fn next_method_frame(&mut self) -> Result<MethodFrame> {
//     let mut frame = self.next_frame().await?;
//
//     loop {
//       match frame {
//         Frame::Method(method)  => {
//           return Ok(method);
//         },
//         // todo: check for the header frame?
//         _ => {
//           frame = self.next_frame().await?;
//         }
//       }
//     }
//   }
//
//   pub async fn next_frame(&mut self) -> Result<Frame> {
//     loop {
//       if let Some(frame) = self.read_frame()? {
//         return Ok(frame);
//       }
//
//       // todo: what if no capacity left?
//       if 0 == self.inner.read_buf(&mut self.buf).await? {
//         // todo: add check for size of the buf, if its error or connection close
//         bail!("Failed to read. Connection closed")
//       }
//     }
//   }
//
//   pub fn read_frame(&mut self) -> Result<Option<Frame>> {
//     if self.buf.len() < FRAME_HEADER_SIZE {
//       debug!("read_frame: frame header is no available");
//       return Ok(None);
//     }
//
//     let mut buf = Cursor::new(&self.buf[..7]);
//     let frame_type = buf.read_byte()?;
//     let chan = buf.read_short()?;
//     let size = buf.read_int()?;
//
//     // header + body_size + frame_end_byte
//     let frame_size = FRAME_HEADER_SIZE + size as usize + FRAME_END_SIZE;
//     if self.buf.len() < frame_size as usize {
//       debug!("read_frame: frame body is no available");
//       return Ok(None)
//     }
//
//     self.buf.advance(7);
//     let body = self.buf.split_to(size as usize).to_vec();
//     // read frame end byte
//     assert_eq!(206, self.buf[0]);
//     self.buf.advance(1);
//
//     debug!("read_frame: Type {}, Chan {}, Size {}", frame_type, chan, size);
//
//     let frame = match frame_type {
//       1 => {
//         let mut meta = Cursor::new(body[..4].to_vec());
//         let class_id = meta.read_short()?;
//         let method_id = meta.read_short()?;
//
//         Frame::Method(MethodFrame { chan, class_id, method_id, body, content_header: None, content_body: None })
//       },
//       2 => {
//         let mut meta = Cursor::new(body[..14].to_vec());
//         let class_id = meta.read_short()?;
//         let _weight = meta.read_short()?;
//         let body_len = meta.read_long()?;
//         let prop_flags = meta.read_short()?;
//
//         Frame::Header(HeaderFrame {
//           chan,
//           class_id,
//           body_len,
//           prop_flags,
//           prop_list: body[14..].to_vec()
//         })
//       }
//       3 => {
//         Frame::Body(BodyFrame {
//           chan,
//           body
//         })
//       }
//       4 => {
//         Frame::Heartbeat
//       },
//       // todo: fix this
//       _ => {
//         panic!("Unknown frame type")
//       }
//     };
//
//     Ok(Some(frame))
//   }
// }

// pub struct FrameWriter {
//   inner: BufWriter<OwnedWriteHalf>
// }
//
// impl FrameWriter {
//   pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
//     Self { inner }
//   }
//
//   pub async fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
//     self.inner.write_all(buf).await?;
//     self.inner.flush().await?;
//     Ok(())
//   }
//
//   pub async fn write_frame<T: TryInto<Vec<u8>, Error=anyhow::Error>>(&mut self, chan: i16, args: T) -> Result<()> {
//     use std::io::Write;
//
//     let arg_buff = args.try_into()?;
//     let mut frame_buff = vec![];
//     frame_buff.write_byte(1)?;
//     frame_buff.write_short(chan)?;
//     Encode::write_uint(&mut frame_buff, arg_buff.len() as u32)?;
//     Write::write(&mut frame_buff, &arg_buff)?;
//     frame_buff.write_byte(0xCE)?;
//     self.write_all(&frame_buff).await?;
//
//     Ok(())
//   }
// }
