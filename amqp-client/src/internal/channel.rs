use std::collections::{HashMap, VecDeque};
use tokio::sync::{oneshot};
use tokio::sync::mpsc::{UnboundedSender};
use crate::api::basic::fields::MessageProperties;
use crate::protocol::types::{Frame, AmqpMessage, ChannelId, ContentHeader, ContentBody, Long};

pub type OneTimeSender = oneshot::Sender<Frame>;
pub type AckSender = oneshot::Sender<()>;

#[derive(Debug)]
pub enum CommandPayload {
  RegisterResponder((ChannelId, oneshot::Sender<Frame>)),
  RegisterChannel((ChannelId, UnboundedSender<AmqpMessage>)),
  RegisterConsumer(ChannelId, String, UnboundedSender<Message>),
}

pub type Command = (CommandPayload, AckSender);

#[derive(Debug)]
pub enum ContentFrame {
  WithMethod(Frame),
  WithContentHeader((Frame, ContentHeader)),
  WithBody((Frame, ContentHeader, ContentBody))
}

impl ContentFrame {
  fn new(method: Frame) -> Self {
    Self::WithMethod(method)
  }

  pub fn with_content_header(self, header: ContentHeader) -> Self {
    if let ContentFrame::WithMethod(frame) = self {
      Self::WithContentHeader((frame, header))
    } else {
      panic!("Invalid state transition")
    }
  }

  pub fn with_body(self, mut body: ContentBody) -> Self {
    match self {
      ContentFrame::WithContentHeader((frame, header)) => {
        Self::WithBody((frame, header, body))
      },
      ContentFrame::WithBody((frame, header, mut curr_body)) => {
        curr_body.0.append(&mut body.0);
        Self::WithBody((frame, header, curr_body))
      },
      _ => {
        panic!("Invalid state transition")
      }
    }
  }

  pub fn is_complete(&self) -> bool {
    match self {
      ContentFrame::WithBody((_,header,body)) => {
        header.body_len <= body.0.len() as Long
      }
      _ => {
        false
      }
    }
  }
}

#[derive(Debug)]
pub struct Message {
  pub properties: MessageProperties,
  pub content: Vec<u8>
}

pub (crate) struct ChannelManager {
  sync_waiters: HashMap<ChannelId, VecDeque<OneTimeSender>>,
  channel_dispatchers: HashMap<ChannelId, UnboundedSender<AmqpMessage>>,
  consumers: HashMap<ChannelId, HashMap<String, UnboundedSender<Message>>>,
}

impl ChannelManager {
  pub fn new() -> Self {
    Self {
      sync_waiters: Default::default(),
      consumers: Default::default(),
      channel_dispatchers: Default::default()
    }
  }

  pub fn get_responder(&mut self, channel: ChannelId) -> OneTimeSender {
    self.sync_waiters.get_mut(&channel).unwrap().pop_front().unwrap()
  }

  pub fn register_responder(&mut self, channel: ChannelId, responder: OneTimeSender) {
    if self.sync_waiters.contains_key(&channel) {
      let channel_waiters = self.sync_waiters.get_mut(&channel).unwrap();
      channel_waiters.push_back(responder);
    } else {
      self.sync_waiters.insert(channel, VecDeque::from([responder]));
    }
  }

  pub fn register_channel(&mut self, channel: ChannelId, incoming_tx: UnboundedSender<AmqpMessage>) {
    self.channel_dispatchers.insert(channel, incoming_tx);
  }

  pub fn register_consumer(&mut self, channel: ChannelId, tag: String, consumer_tx: UnboundedSender<Message>) {
    if !self.consumers.contains_key(&channel) {
      self.consumers.insert(channel, Default::default());
    }

    let channel_consumers = self.consumers.get_mut(&channel).unwrap();
    channel_consumers.insert(tag, consumer_tx);
  }

  pub fn dispatch_content_frame(&mut self, channel: ChannelId, frame: ContentFrame) {
    if let ContentFrame::WithBody((frame, header, body)) = frame {
      let channel_consumers = self.consumers.get_mut(&channel).unwrap();

      match frame {
        Frame::BasicDeliver(deliver) => {
          let consumer = channel_consumers.get_mut(&deliver.consumer_tag.0).unwrap();
          // todo: add metadata to the message
          let message = Message {
            properties: header.prop_list,
            content: body.0
          };

          consumer.send(message).unwrap();
        },
        _ => {
          todo!("to be implemented")
        }
      }

    } else {
      panic!("Invalid frame variant")
    }
  }
}
