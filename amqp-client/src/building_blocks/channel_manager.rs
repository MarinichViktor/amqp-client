use std::collections::{HashMap, VecDeque};
use tokio::sync::{oneshot};
use tokio::sync::mpsc::{UnboundedSender};
use crate::protocol::types::{ChannelId};
use crate::protocol::frame::{FrameEnvelope, Frame, ContentFrame};
use crate::protocol::message::Message;

pub (crate) struct ChannelManager {
  sync_waiters: HashMap<ChannelId, VecDeque<oneshot::Sender<Frame>>>,
  channel_dispatchers: HashMap<ChannelId, UnboundedSender<FrameEnvelope>>,
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

  pub fn get_responder(&mut self, channel: ChannelId) -> oneshot::Sender<Frame> {
    self.sync_waiters.get_mut(&channel).unwrap().pop_front().unwrap()
  }

  pub fn register_responder(&mut self, channel: ChannelId, responder: oneshot::Sender<Frame>) {
    if self.sync_waiters.contains_key(&channel) {
      let channel_waiters = self.sync_waiters.get_mut(&channel).unwrap();
      channel_waiters.push_back(responder);
    } else {
      self.sync_waiters.insert(channel, VecDeque::from([responder]));
    }
  }

  pub fn register_channel(&mut self, channel: ChannelId, incoming_tx: UnboundedSender<FrameEnvelope>) {
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
