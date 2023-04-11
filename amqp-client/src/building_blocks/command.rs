use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use crate::protocol::frame::{FrameEnvelope, Frame};
use crate::protocol::message::Message;
use crate::protocol::types::ChannelId;

#[derive(Debug)]
pub enum CommandPayload {
  RegisterResponder((ChannelId, oneshot::Sender<Frame>)),
  RegisterChannel((ChannelId, UnboundedSender<FrameEnvelope>)),
  RegisterConsumer(ChannelId, String, UnboundedSender<Message>),
}

pub type Command = (CommandPayload, oneshot::Sender<()>);
