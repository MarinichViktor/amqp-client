use std::collections::HashMap;
use std::fmt::format;
use std::io::{BufRead, Cursor, Error};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use anyhow::bail;
use bytes::{Buf, BufMut, BytesMut};
use log::{debug, info};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf};
use tokio::net::TcpStream;
use url::Url;
use amqp_protocol::dec::Decode;
use amqp_protocol::types::Property;
use crate::protocol::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT};
use crate::protocol::frame::{BodyFrame, Frame, HeaderFrame, MethodFrame};
use crate::protocol::stream::{AmqpStream};
use crate::{Result, Channel, Connection};
use crate::protocol::connection;
use crate::utils::IdAllocator;

pub mod constants;
pub mod methods;

pub struct ConnectionFactory;

impl ConnectionFactory {
  pub async fn create(uri: &str) -> Result<AmqConnection> {
    let options = Self::parse_uri(uri)?;
    let stream = TcpStream::connect(format!("{}:{}", options.host, options.port)).await?;

    Ok(AmqConnection::new(stream, options))
  }

  fn parse_uri(uri: &str) -> Result<ConnectionOpts> {
    let url = Url::parse(uri).unwrap();
    let host = if url.has_host() {
      url.host().unwrap().to_string()
    } else {
      String::from("localhost")
    };
    let port = url.port().unwrap_or_else(|| 5672);
    let (login, password) = if url.has_authority() {
      (url.username().to_string(), url.password().unwrap().to_string())
    } else {
      bail!("Provide username and password in the connection url");
    };

    Ok(ConnectionOpts {
      host,
      port,
      login,
      password,
      vhost: url.path()[1..].into(),
    })
  }
}

pub struct AmqConnection {
  reader: Arc<FrameReader>,
  writer: Arc<FrameWriter>,
  options: ConnectionOpts,
  // channels: Arc<Mutex<HashMap<i16,Arc<Channel>>>>,
  // id_allocator: IdAllocator,
  // pending: Arc<Mutex<HashMap<i16, AmqMethodFrame>>>
}

#[derive(Clone)]
pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String,
  pub vhost: String,
}

impl AmqConnection {
  pub fn new(stream: TcpStream, options: ConnectionOpts) -> Self {
    let stream_parts = stream.into_split();
    let reader = Arc::new(FrameReader::new(BufReader::new(stream_parts.0)));
    let writer = Arc::new(FrameWriter::new(BufWriter::new(stream_parts.1)));

    Self {
      reader,
      writer,
      options,
    }
  }

  pub async fn connect(&mut self) -> Result<()> {
    use crate::protocol::connection::constants::PROTOCOL_HEADER;
    use crate::protocol::connection::methods as conn_methods;

    // let stream_parts = stream.into_split();
    // let reader = Arc::new(FrameReader::new(BufReader::new(stream_parts.0)));
    // let writer = Arc::new(FrameWriter::new(BufWriter::new(stream_parts.1)));

    // let conn =  AmqConnection {
    //   reader,
    //   writer,
    //   buff: BytesMut::with_capacity( 4096),
      // channels: Arc::new(Mutex::new(HashMap::new())),
      // id_allocator: IdAllocator::new(),
      // options,
      // pending: Arc::new(Mutex::new(HashMap::new()))
    // };

    // info!("Connecting to the server");
    // let mut reader = conn.reader;
    // let mut writer = conn.writer;

    info!("Sending ProtocolHeader");
    self.writer.write_all(&PROTOCOL_HEADER).await?;

    let frame = self.reader.inner.next_method_frame()?;
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
      locale: DEFAULT_LOCALE.to_string()
    };
    info!("Sending StartOk");
    writer.invoke(0, start_ok_method)?;

    let frame = reader.next_method_frame()?;
    let tune_method: conn_methods::Tune = frame.body.try_into()?;

    // todo: use values from conn options
    let tune_ok_method = conn_methods::TuneOk {
      chan_max: tune_method.chan_max,
      frame_max: tune_method.frame_max,
      heartbeat: tune_method.heartbeat
    };
    info!("Sending TuneOk");
    writer.invoke(0, tune_ok_method)?;

    let open_method = conn_methods::Open {
      vhost: self.options.vhost.clone(),
      ..conn_methods::Open::default()
    };
    info!("Sending OpenMethod");
    writer.invoke(0, open_method)?;
    let frame = reader.next_method_frame()?;
    let _open_ok_method: conn_methods::OpenOk = frame.body.try_into()?;
    info!("Connected to the server");

    drop(reader);
    drop(writer);

    self.start_listener();
    Ok(())
  }

  pub fn next_frame(&mut self) -> Result<Option<Frame>> {
    if self.buff.len() < 7 {
      return Ok(None);
    }

    let mut buf = Cursor::new(&self.buff[..7]);
    // let mut frame_header = self.read_cursor(7)?;
    let frame_type = buf.read_byte()?;
    let chan = buf.read_short()?;
    let size = buf.read_int()?;

    // header + body_size + frame_end_byte
    let frame_size = 7 +size + 1;
    if self.buff.len() < frame_size as usize {
      return Ok(None)
    }

    self.buff.advance(7);
    let body = self.buff.split_to(size as usize).to_vec();
    // read frame end byte
    assert_eq!(206, self.buff[0]);
    self.buff.advance(1);

    info!("next_frame frame_type {}, chan {}, size {}, body {:?}", frame_type, chan, size, body);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::Method(MethodFrame { chan, class_id, method_id, body, content_header: None, content_body: None })
      },
      2 => {
        let mut meta = Cursor::new(body[..14].to_vec());
        let class_id = meta.read_short()?;
        let _weight = meta.read_short()?;
        let body_len = meta.read_long()?;
        let prop_flags = meta.read_short()?;

        Frame::Header(HeaderFrame {
          chan,
          class_id,
          body_len,
          prop_flags,
          prop_list: body[14..].to_vec()
        })
      }
      3 => {
        info!("Body frame {:?}, len {}", &body, body.len());
        Frame::Body(BodyFrame {
          chan,
          body
        })
      }
      4 => {
        Frame::Heartbeat
      },
      // todo: fix this
      _ => {
        panic!("Unknown frame type")
      }
    };

    Ok(Some(frame))
  }
/*  pub fn create_channel(&mut self) -> Result<Arc<Channel>> {
    let id = self.id_allocator.allocate();
    info!("Creating channel with id: {}", id);
    let chan = Channel::new(id, self.amqp_stream.clone());
    let chan = Arc::new(chan);
    self.channels.lock().unwrap().insert(chan.id, chan.clone());
    chan.open()?;

    Ok(chan)
  }

  pub fn start_listener(&mut self) {
    let reader = self.amqp_stream.reader.clone();
    let channels = self.channels.clone();
    let pending = self.pending.clone();

    std::thread::spawn(move || {
      let mut reader = reader.lock().unwrap();

      while let Ok(frame) = reader.next_frame() {
        match frame {
          AmqFrame::Method(method_frame) => {
            debug!("Received method frame: channel {}, class_id {}, method_id: {}", method_frame.chan, method_frame.class_id, method_frame.method_id);
            if method_frame.has_content() {
              pending.lock().unwrap().insert(method_frame.chan, method_frame);
            } else {
              channels.lock().unwrap()[&method_frame.chan].handle_frame(method_frame).unwrap();
            }
          },
          AmqFrame::Header(header) => {
            let chan = header.chan;
            debug!("Received header frame: channel {}, class_id {}", header.chan, header.class_id);
            pending.lock().unwrap().get_mut(&chan).unwrap().content_header = Some(header);
          },
          AmqFrame::Body(mut frame) => {
            debug!("Received body frame");
            let mut pending_frames = pending.lock().unwrap();

            let mut partial_frame = pending_frames.remove(&frame.chan).unwrap();

            let mut content_body = partial_frame.content_body.take().unwrap_or_else(|| vec![]);
            content_body.append(&mut frame.body);

            let curr_body_len = content_body.len();
            let expected_body_len = match &partial_frame.content_header {
              Some(x) => x.body_len,
              _ => panic!("failed to get content header")
            };
            partial_frame.content_body = Some(content_body);

            if curr_body_len as i64 == expected_body_len {
              debug!("Received full frame body");
              channels.lock().unwrap()[&frame.chan].handle_frame(partial_frame).unwrap();
            } else {
              debug!("Received {} bytes, expected {}. Waiting on the next frames", curr_body_len, expected_body_len);
              pending_frames.insert(frame.chan, partial_frame);
            }
          }
          _ => {
            // panic!("Unsupported frame type")
          }
        }
      }
    });
  }*/
}

const FRAME_HEADER_SIZE: usize = 7;
const FRAME_END_SIZE: usize = 1;

pub struct FrameReader {
  inner: BufReader<OwnedReadHalf>,
  buf: BytesMut,
}

impl FrameReader {
  pub fn new(inner: BufReader<OwnedReadHalf>) -> Self {
    Self {
      inner,
      // todo: review default capacity
      buf: BytesMut::with_capacity(128 * 1024)
    }
  }

  pub fn next_frame(&mut self) -> Result<Option<Frame>> {
    if self.buf.len() < FRAME_HEADER_SIZE {
      return Ok(None);
    }

    let mut buf = Cursor::new(&self.buf[..7]);
    let frame_type = buf.read_byte()?;
    let chan = buf.read_short()?;
    let size = buf.read_int()?;

    // header + body_size + frame_end_byte
    let frame_size = FRAME_HEADER_SIZE + size as usize + FRAME_END_SIZE;
    if self.buf.len() < frame_size as usize {
      return Ok(None)
    }

    self.buf.advance(7);
    let body = self.buf.split_to(size as usize).to_vec();
    // read frame end byte
    assert_eq!(206, self.buf[0]);
    self.buf.advance(1);

    info!("next_frame frame_type {}, chan {}, size {}, body {:?}", frame_type, chan, size, body);

    let frame = match frame_type {
      1 => {
        let mut meta = Cursor::new(body[..4].to_vec());
        let class_id = meta.read_short()?;
        let method_id = meta.read_short()?;

        Frame::Method(MethodFrame { chan, class_id, method_id, body, content_header: None, content_body: None })
      },
      2 => {
        let mut meta = Cursor::new(body[..14].to_vec());
        let class_id = meta.read_short()?;
        let _weight = meta.read_short()?;
        let body_len = meta.read_long()?;
        let prop_flags = meta.read_short()?;

        Frame::Header(HeaderFrame {
          chan,
          class_id,
          body_len,
          prop_flags,
          prop_list: body[14..].to_vec()
        })
      }
      3 => {
        info!("Body frame {:?}, len {}", &body, body.len());
        Frame::Body(BodyFrame {
          chan,
          body
        })
      }
      4 => {
        Frame::Heartbeat
      },
      // todo: fix this
      _ => {
        panic!("Unknown frame type")
      }
    };

    Ok(Some(frame))
  }
}

pub struct FrameWriter {
  inner: BufWriter<OwnedWriteHalf>
}

impl FrameWriter {
  pub fn new(inner: BufWriter<OwnedWriteHalf>) -> Self {
    Self { inner }
  }

  pub async fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Result<()> {
    self.inner.write_all(buf).await?;
    Ok(())
  }
}
