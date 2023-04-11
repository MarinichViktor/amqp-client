use std::io::Cursor;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use crate::protocol::dec::Decode;
use crate::protocol::enc::Encode;
use crate::protocol::frame::{BasicAck, BasicReject, FrameEnvelope};
use crate::protocol::types::{ChannelId, PropTable};
use crate::Result;

#[derive(Debug)]
pub struct Message {
  channel: ChannelId,
  outgoing_tx: UnboundedSender<FrameEnvelope>,
  pub properties: MessageProperties,
  pub content: Vec<u8>
}

impl Message {
  pub fn ack(&self, delivery_tag: i64, multiple: bool) -> Result<()> {
    let method = BasicAck { delivery_tag, multiple };
    self.outgoing_tx.send((self.channel, method.into_frame()))?;
    Ok(())
  }

  pub fn reject(&self, delivery_tag: i64, requeue: bool) -> Result<()> {
    let method = BasicReject { delivery_tag, requeue };
    self.outgoing_tx.send((self.channel, method.into_frame()))?;
    Ok(())
  }
}


#[derive(Debug)]
pub enum MessageDeliveryMode {
  Persistent,
  NonPersistent
}

#[derive(Default, Debug)]
pub struct MessageProperties {
  pub content_type: Option<String>,
  pub content_encoding: Option<String>,
  pub headers: Option<PropTable>,
  pub delivery_mode: Option<MessageDeliveryMode>,
  pub priority: Option<u8>,
  pub correlation_id: Option<String>,
  pub reply_to: Option<String>,
  pub expiration: Option<String>,
  pub message_id: Option<String>,
  pub timestamp: Option<Duration>,
  pub ty: Option<String>,
  pub user_id: Option<String>,
  pub app_id: Option<String>,
  reserved: String
}

impl MessageProperties {
  pub fn new() -> Self {
    Default::default()
  }
}

impl Into<Vec<u8>> for MessageProperties {
  fn into(self) -> Vec<u8> {
    let mut result = vec![];
    let mut flag = 0_u16;
    let mut value = vec![];

    if let Some(content_type) = self.content_type {
      flag = flag | 0b1000_0000_0000_0000;
      value.write_shortstr(content_type.into()).unwrap();
    }

    if let Some(content_encoding) = self.content_encoding {
      flag = flag | 0b100_0000_0000_0000;
      value.write_shortstr(content_encoding.into()).unwrap();
    }

    if let Some(headers) = self.headers {
      flag = flag | 0b10_0000_0000_0000;
      value.write_proptable(headers).unwrap();
    }

    if let Some(delivery_mode) = self.delivery_mode {
      flag = flag | 0b1_0000_0000_0000;
      match delivery_mode {
        MessageDeliveryMode::NonPersistent => {
          value.write_byte(1).unwrap();
        }
        MessageDeliveryMode::Persistent => {
          value.write_byte(2).unwrap();
        }
      }
    }

    if let Some(priority) = self.priority {
      flag = flag | 0b1000_0000_0000;
      value.write_byte(priority).unwrap();
    }

    if let Some(correlation_id) = self.correlation_id {
      flag = flag | 0b100_0000_0000;
      value.write_shortstr(correlation_id.into()).unwrap();
    }

    if let Some(reply_to) = self.reply_to {
      flag = flag | 0b10_0000_0000;
      value.write_shortstr(reply_to.into()).unwrap();
    }

    if let Some(expiration) = self.expiration {
      flag = flag | 0b1_0000_0000;
      value.write_shortstr(expiration.into()).unwrap();
    }

    if let Some(message_id) = self.message_id {
      flag = flag | 0b1000_0000;
      value.write_shortstr(message_id.into()).unwrap();
    }

    if let Some(timestamp) = self.timestamp {
      flag = flag | 0b100_0000;
      value.write_ulong(timestamp.as_secs()).unwrap();
    }

    if let Some(ty) = self.ty {
      flag = flag | 0b10_0000;
      value.write_shortstr(ty.into()).unwrap();
    }


    if let Some(user_id) = self.user_id {
      flag = flag | 0b1_0000;
      value.write_shortstr(user_id.into()).unwrap();
    }

    if let Some(app_id) = self.app_id {
      flag = flag | 0b1000;
      value.write_shortstr(app_id.into()).unwrap();
    }

    result.write_ushort(flag).unwrap();
    result.append(&mut value);

    result
  }
}

impl From<Vec<u8>> for MessageProperties {
  fn from(mut data: Vec<u8>) -> Self {
    let mut cursor = Cursor::new(data);
    let flag = cursor.read_ushort().unwrap();
    let mut fields = MessageProperties::new();

    if (flag & 0b1000_0000_0000_0000 ) != 0 {
      fields.content_type = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b100_0000_0000_0000 ) != 0 {
      fields.content_encoding = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b10_0000_0000_0000 ) != 0 {
      fields.headers = Some(cursor.read_proptable().unwrap());
    }

    if (flag & 0b1_0000_0000_0000 ) != 0 {
      let mode = cursor.read_byte().unwrap();

      fields.delivery_mode = Some(if mode == 2 {
        MessageDeliveryMode::Persistent
      } else {
        MessageDeliveryMode::NonPersistent
      });

      fields.headers = Some(cursor.read_proptable().unwrap());
    }

    if (flag & 0b1000_0000_0000 ) != 0 {
      fields.priority = Some(cursor.read_byte().unwrap());
    }

    if (flag & 0b100_0000_0000 ) != 0 {
      fields.correlation_id = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b10_0000_0000 ) != 0 {
      fields.reply_to = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b1_0000_0000 ) != 0 {
      fields.expiration = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b1000_0000 ) != 0 {
      fields.message_id = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b100_0000 ) != 0 {
      fields.timestamp = Some(Duration::from_secs(cursor.read_ulong().unwrap()));
    }

    if (flag & 0b10_0000 ) != 0 {
      fields.ty = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b1_0000 ) != 0 {
      fields.user_id = Some(cursor.read_shortstr().unwrap().0);
    }

    if (flag & 0b1000 ) != 0 {
      fields.app_id = Some(cursor.read_shortstr().unwrap().0);
    }

    fields
  }
}
