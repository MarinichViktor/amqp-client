use log::info;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::protocol::types::{ChannelId};
use crate::{Result};
use crate::protocol::frame::{FrameEnvelope, Frame};
use crate::protocol::frame::ConnectionCloseOk;

pub struct DefaultAmqChannel {
  pub id: ChannelId,
  outgoing_tx: UnboundedSender<FrameEnvelope>,
}

impl DefaultAmqChannel {
  pub fn open(
    outgoing_tx: UnboundedSender<FrameEnvelope>,
    incoming_rx: UnboundedReceiver<FrameEnvelope>,
    close_tx: broadcast::Sender<()>,
  ) -> Result<Self> {
    let channel = Self { id: 0, outgoing_tx };
    channel.spawn_incoming_msg_handler(incoming_rx, close_tx);

    Ok(channel)
  }

  fn spawn_incoming_msg_handler(&self, mut incoming_rx: UnboundedReceiver<FrameEnvelope>, close_tx: broadcast::Sender<()>) {
    let outgoing_tx = self.outgoing_tx.clone();
    tokio::spawn(async move {
      while let Some((_, frame)) = incoming_rx.recv().await {
        match frame {
          Frame::ConnectionClose(connection_close) => {
            info!("Connection closed with code: {}, reason: {}", connection_close.reply_code, connection_close.reply_text.0);
            outgoing_tx.send((0, ConnectionCloseOk {}.into_frame())).unwrap();
            close_tx.send(()).unwrap();
            break;
          },
          Frame::ConnectionCloseOk(_) => {
            info!("connection close-ok received");
            close_tx.send(()).unwrap();
            break;
          }
          _ => {
            todo!("Implement handler")
          }
        }
      }
    });
  }
}
