use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use amqp_protocol::enc::Encode;
use amqp_protocol::types::AmqpMethodArgs;
use crate::protocol::basic::fields::Fields;
use crate::protocol::frame::{HeaderFrame, MethodFrame};

#[derive(Debug)]
pub struct RawFrame {
  pub ch: i16,
  pub cid: i16,
  pub mid: i16,
  pub args: Vec<u8>,
  pub prop_fields: Option<Fields>,
  pub body: Option<Vec<u8>>
}

impl RawFrame {
  pub fn new(ch: i16, cid: i16, mid: i16, args: Vec<u8>, prop_fields: Option<Fields>, body: Option<Vec<u8>>) -> Self {
    Self { ch, cid, mid, args, prop_fields, body }
  }
}

impl Into<Vec<Vec<u8>>> for RawFrame {
  fn into(self) -> Vec<Vec<u8>> {
    let mut result = vec![];

    let mut frame_buff = vec![];
    frame_buff.write_byte(1).unwrap();
    frame_buff.write_short(self.ch).unwrap();
    Encode::write_uint(&mut frame_buff, (self.args.len() + 4)  as u32).unwrap();
    frame_buff.write_short(self.cid).unwrap();
    frame_buff.write_short(self.mid).unwrap();
    Write::write(&mut frame_buff, &self.args).unwrap();
    frame_buff.write_byte(0xCE).unwrap();
    result.push(frame_buff);

    if let Some(mut body) = self.body {
      let mut header_body = vec![];
      header_body.write_short(self.cid).unwrap();
      header_body.write_short(0).unwrap();
      header_body.write_long(body.len() as i64).unwrap();

      let mut fields = Fields::new();
      fields.user_id = Some("qrhorpow".into());
      fields.content_type = Some("application/json".into());
      fields.ty = Some("my-msg".into());
      fields.content_encoding = Some("text/plain".into());
      fields.reply_to = Some("some-q".into());
      fields.timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
      fields.message_id = Some("123123".into());
      // todo: allow fields passing instead of hardcoded value
      // header_body.write_short(0).unwrap();
      header_body.append(&mut fields.into());
      // header_body.write_short(-32768).unwrap();
      // header_body.write_short_str("text/plain".into()).unwrap();

      let mut header_frame = vec![];
      header_frame.write_byte(2).unwrap();
      header_frame.write_short(self.ch).unwrap();
      header_frame.write_int(header_body.len() as i32).unwrap();
      header_frame.append(&mut header_body);
      header_frame.write_byte(0xCE).unwrap();
      result.push(header_frame);

      // todo: split into multiple frames based on max-frame-size
      let mut body_frame = vec![];
      body_frame.write_byte(3).unwrap();
      body_frame.write_short(self.ch).unwrap();
      body_frame.write_int(body.len() as i32).unwrap();
      body_frame.append(&mut body);
      body_frame.write_byte(0xCE).unwrap();

      result.push(body_frame);
    }

    result
  }
}

pub struct Frame2<T: AmqpMethodArgs> {
  pub ch: i16,
  pub args: T,
  pub prop_fields: Option<Vec<u8>>,
  pub body: Option<Vec<u8>>
}

impl <T: AmqpMethodArgs> Frame2<T> {
  pub fn new(ch: i16, args: T) -> Self {
    Self {
      ch,
      args,
      prop_fields: None,
      body: None
    }
  }
}

impl <T: AmqpMethodArgs> Into<Frame2<T>> for RawFrame {
  fn into(self) -> Frame2<T> {
    Frame2::new(self.ch, self.args.try_into().unwrap())
  }
}


impl <T: AmqpMethodArgs> Into<RawFrame> for Frame2<T> {
  fn into(self) -> RawFrame {
    RawFrame {
      ch: self.ch,
      cid: self.args.class_id(),
      mid: self.args.method_id(),
      args: self.args.try_into().unwrap(),
      prop_fields: None,
      body: self.body
    }
  }
}

pub struct PendingFrame {
  method: MethodFrame,
  header: Option<HeaderFrame>,
  body: Vec<u8>
}

impl PendingFrame {
  pub fn new(method: MethodFrame) -> Self {
    Self {
      method,
      header: None,
      body: vec![]
    }
  }

  pub fn header(&mut self, header: HeaderFrame) {
    self.header = Some(header);
  }

  pub fn append_body(&mut self, mut data: &mut Vec<u8>) {
    self.body.append(&mut data);
  }

  pub fn is_completed(&self) -> bool {
    if self.header.is_none() {
      return false;
    }

    return self.body.len() as i64 == self.header.as_ref().unwrap().body_len;
  }
}

impl Into<RawFrame> for PendingFrame {
  fn into(self) -> RawFrame {
    let prop_fields = if let Some(header) = self.header {
      Some(Fields::from(header.prop_list))
    } else {
      None
    };

    let body = if !self.body.is_empty() {
      Some(self.body)
    } else {
      None
    };


    RawFrame {
      ch: self.method.chan,
      cid: self.method.class_id,
      mid: self.method.method_id,
      args: self.method.body,
      prop_fields,
      body
    }
  }
}
