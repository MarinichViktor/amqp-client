use std::collections::{HashMap, VecDeque};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{UnboundedSender};
use crate::protocol::types::{Frame, AmqpMessage, ChannelId};

pub type OneTimeSender = oneshot::Sender<Frame>;
pub type AckSender = oneshot::Sender<()>;

#[derive(Debug)]
pub enum CommandPayload {
  RegisterResponder((ChannelId, oneshot::Sender<Frame>)),
  RegisterChannel((ChannelId, mpsc::UnboundedSender<AmqpMessage>)),
  RegisterConsumer,
}

pub type Command = (CommandPayload, AckSender);
pub (crate) struct ChannelManager {
  sync_waiters: HashMap<ChannelId, VecDeque<OneTimeSender>>,
  channel_dispatchers: HashMap<ChannelId, UnboundedSender<AmqpMessage>>,
  consumers: HashMap<ChannelId, VecDeque<OneTimeSender>>,
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
}
