use std::collections::HashMap;
use std::sync::{Arc};

use log::{debug, info};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};

use amqp_protocol::types::Property;

use crate::{Channel, Result};
use crate::protocol::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::frame::{Frame, HeaderFrame, MethodFrame};
use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;
use crate::utils::IdAllocator;
use crate::protocol::connection::options::ConnectionOpts;

pub mod constants;
pub mod methods;
pub mod factory;
pub mod options;


#[derive(Debug)]
pub struct MethodRequest {
  channel: i16,
  payload: Vec<u8>
}

#[derive(Clone)]
pub struct FrameSender(mpsc::Sender<MethodRequest>);
pub struct FrameReceiver(mpsc::Receiver<MethodRequest>);

impl FrameSender {
  pub async fn send<T>(&mut self, channel: i16, request: T) -> Result<()>
    where T: TryInto<Vec<u8>, Error = crate::Error>
  {
    self.0.send(MethodRequest {
      channel,
      payload: request.try_into()?
    }).await?;
    Ok(())
  }
}

pub struct Connection {
  reader: Option<FrameReader>,
  writer: Arc<Mutex<FrameWriter>>,
  options: ConnectionOpts,
  id_allocator: IdAllocator,
  channels: Arc<Mutex<HashMap<i16, mpsc::Sender<MethodFrame>>>>,
  sender: FrameSender,
  receiver: Option<FrameReceiver>
  // pending: Arc<Mutex<HashMap<i16, AmqMethodFrame>>>
}


impl Connection {
  pub fn new(stream: TcpStream, options: ConnectionOpts) -> Self {
    let stream_parts = stream.into_split();
    let reader = Some(FrameReader::new(BufReader::new(stream_parts.0)));
    let writer = Arc::new(Mutex::new(FrameWriter::new(BufWriter::new(stream_parts.1))));
    let (sender, receiver) = mpsc::channel(128);

    Self {
      reader,
      writer,
      options,
      id_allocator: IdAllocator::new(),
      channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
      sender: FrameSender(sender),
      receiver: Some(FrameReceiver(receiver))
    }
  }

  pub async fn connect(&mut self) -> Result<()> {
    use crate::protocol::connection::constants::PROTOCOL_HEADER;
    use crate::protocol::connection::methods as conn_methods;

    info!("Connecting to the server");
    let mut writer = self.writer.lock().await;
    let reader = self.reader.as_mut().unwrap();

    info!("Sending [ProtocolHeader]");
    writer.write_all(&PROTOCOL_HEADER).await?;

    let frame = reader.next_method_frame().await?;
    let _start_method: conn_methods::Start = frame.body.try_into()?;

    let client_properties = HashMap::from([
      ("product".to_string(), Property::LongStr(PRODUCT.to_string())),
      ("platform".to_string(), Property::LongStr(PLATFORM.to_string())),
      ("copyright".to_string(), Property::LongStr(COPYRIGHT.to_string())),
      ("information".to_string(), Property::LongStr(INFORMATION.to_string()))
    ]);
    let start_ok_method = conn_methods::StartOk {
      properties: client_properties,
      mechanism: DEFAULT_AUTH_MECHANISM.to_string(),
      response: format!("\x00{}\x00{}", self.options.login.as_str(), self.options.password),
      locale: DEFAULT_LOCALE.to_string(),
    };
    info!("Sending [StartOk]");
    // todo: add const for default channel or separate struct
    writer.write_method_frame(0, start_ok_method.try_into()?).await?;

    let frame = reader.next_method_frame().await?;
    let tune_method: conn_methods::Tune = frame.body.try_into()?;

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.chan_max,
      frame_max: tune_method.frame_max,
      heartbeat: tune_method.heartbeat,
    };
    info!("Sending [TuneOk]");
    writer.write_method_frame(0, tune_ok_method.try_into()?).await?;

    let open_method = conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    };

    info!("Sending [OpenMethod]");
    writer.write_method_frame(0, open_method.try_into()?).await?;

    let frame = reader.next_method_frame().await?;
    let _open_ok_method: conn_methods::OpenOk = frame.body.try_into()?;
    drop(writer);

    info!("Connected to the server");

    self.start_listener();

    Ok(())
  }

  pub fn start_listener(&mut self) {
    let mut reader = self.reader.take().unwrap();
    let channels = self.channels.clone();

    // let reader = self.amqp_stream.reader.clone();
    // let pending = self.pending.clone();
    let handle = Handle::current();
    let mut receiver = self.receiver.take().unwrap();
    let writer = self.writer.clone();

    std::thread::spawn(move || {
      let future1 = handle.spawn(async move {
        println!("Waiting for a frame");
        while let Ok(frame) = reader.next_frame().await {
          println!("Received frame {:?}", frame);
          match frame {
            Frame::Method(method_frame) => {
              debug!("Received method frame: channel {}, class_id {}, method_id: {}", method_frame.chan, method_frame.class_id, method_frame.method_id);
              if method_frame.has_content() {
                // todo: to be deifend
                println!("Method with content received");
                panic!("Method with content received");
                // pending.lock().unwrap().insert(method_frame.chan, method_frame);
              } else {
                println!("Wait for lock");
                let channels_map = channels.lock().await;
                println!("Send frame to the channel");
                channels_map[&method_frame.chan].send(method_frame).await.unwrap();
                println!("Send frame to the channel 2");
              }
            },
            Frame::Header(header) => {
              let chan = header.chan;
              debug!("Received header frame: channel {}, class_id {}", header.chan, header.class_id);
              // pending.lock().unwrap().get_mut(&chan).unwrap().content_header = Some(header);
            },
            Frame::Body(mut frame) => {
              debug!("Received body frame");
              // let mut pending_frames = pending.lock().unwrap();
              //
              // let mut partial_frame = pending_frames.remove(&frame.chan).unwrap();
              //
              // let mut content_body = partial_frame.content_body.take().unwrap_or_else(|| vec![]);
              // content_body.append(&mut frame.body);
              //
              // let curr_body_len = content_body.len();
              // let expected_body_len = match &partial_frame.content_header {
              //   Some(x) => x.body_len,
              //   _ => panic!("failed to get content header")
              // };
              // partial_frame.content_body = Some(content_body);
              //
              // if curr_body_len as i64 == expected_body_len {
              //   debug!("Received full frame body");
              //   channels.lock().unwrap()[&frame.chan].handle_frame(partial_frame).unwrap();
              // } else {
              //   debug!("Received {} bytes, expected {}. Waiting on the next frames", curr_body_len, expected_body_len);
              //   pending_frames.insert(frame.chan, partial_frame);
              // }
            }
            _ => {
              panic!("Unsupported frame type")
            }
          }
        }

        println!("Received something else");
      });

      let future2 = handle.spawn(async move {
        while let Some(request) = receiver.0.recv().await {
          let mut w = writer.lock().await;
          w.write_method_frame(request.channel, request.payload).await.unwrap();
          println!("Received request");
        }
        println!("Method listener exited");
      });
    });
  }

  pub async fn create_channel(&mut self) -> Result<Channel> {
    let id = self.id_allocator.allocate();

    info!("[Connection] create_channel {}", id);
    let mut channel = Channel::new(id, self.sender.clone());
    info!("Wait for lock ");
    let mut channels = self.channels.lock().await;
    channels.insert(channel.id, channel.inner_tx.clone());
    drop(channels);
    info!("Received lock");
    channel.open().await?;
    info!("Received lock");

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
