use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

use tokio::sync::{mpsc, Mutex, oneshot};
use crate::protocol::frame2::{FrameKind, RawFrame};

use crate::protocol::reader::FrameReader;
use crate::protocol::writer::FrameWriter;

pub struct AmqpConnection {
  reader: Option<FrameReader>,
  writer: Arc<Mutex<FrameWriter>>,
  listeners: Arc<Mutex<HashMap<i16, mpsc::Sender<RawFrame>>>>,
  sync_waiter_queue: Arc<Mutex<HashMap<i16, Vec<oneshot::Sender<RawFrame>>>>>
}

impl AmqpConnection {
  pub fn new(reader: FrameReader, writer: FrameWriter) -> Self {
    Self {
      reader: Some(reader),
      writer: Arc::new(Mutex::new(writer)),
      listeners: Default::default(),
      sync_waiter_queue: Default::default()
    }
  }

  pub fn get_writer(&self) -> Arc<Mutex<FrameWriter>> {
    self.writer.clone()
  }

  pub async fn subscribe(&self, ch: i16) -> mpsc::Receiver<RawFrame> {
    let (tx, rx) = mpsc::channel(16);
    let mut listener = self.listeners.lock().await;
    listener.insert(ch, tx);
    rx
  }

  pub fn start_frames_listener(&mut self) {
    let mut reader = self.reader.take().unwrap();
    let channels = self.listeners.clone();
    let sync_waiter_queue = self.sync_waiter_queue.clone();
    let handle = Handle::current();

    std::thread::spawn(move || {
      handle.spawn(async move {
        while let Ok(frame) = reader.next_frame().await {
          match frame {
            FrameKind::Method(frame) => {
              let mut sync_waiter_queue = sync_waiter_queue.lock().await;

              if let Some(senders) = sync_waiter_queue.get_mut(&frame.ch) {
                if !senders.is_empty() {
                  let sender = senders.remove(0);
                  sender.send(frame).unwrap();
                  continue;
                }
              }

              let mut channels = channels.lock().await;

              if let Some(sender) = channels.get_mut(&frame.ch) {
                sender.send(frame).await.unwrap();
              }
            }
            FrameKind::Heartbeat => {
              // todo: monitor frame frequency
              println!("heartbeat received");
            }
          }
        }
      });
    });
  }
}
