use std::collections::HashMap;
use std::sync::{Arc};
use log::{info};
use tokio::sync::{mpsc, Mutex,oneshot};
use amqp_protocol::enc::Encode;
use crate::protocol::frame2::{Frame2, RawFrame};
use amqp_protocol::types::{Property};
use Property::LongStr;
use crate::connection::constants::{COPYRIGHT, DEFAULT_AUTH_MECHANISM, DEFAULT_LOCALE, INFORMATION, PLATFORM, PRODUCT, PROTOCOL_HEADER};
use crate::connection::options::ConnectionOpts;
use crate::protocol::writer::FrameWriter;

pub struct DefaultChannel {
  pub id: i16,
  writer: Arc<Mutex<FrameWriter>>,
  channel_notifier: Option<mpsc::Receiver<RawFrame>>,
}

impl DefaultChannel {
  pub(crate) fn new(writer: Arc<Mutex<FrameWriter>>, channel_notifier: mpsc::Receiver<RawFrame>) -> Self {

    Self {
      id: 0,
      writer,
      channel_notifier: Some(channel_notifier),
    }
  }

  pub async fn open(&mut self, options: ConnectionOpts) {
    let mut inner_rx = self.channel_notifier.take().unwrap();

    let writer = self.writer.clone();
    let (tx, rx) = oneshot::channel::<()>();

    info!("[Channel] start incoming listener");
    tokio::spawn(async move {
      use crate::connection::methods as conn_methods;

      let mut tx = Some(tx);
      info!("Sending [ProtocolHeader]");
      {
        let mut writer = writer.lock().await;
        writer.write_binary(&PROTOCOL_HEADER).await.unwrap();
      }

      loop {
        if let Some(frame) = inner_rx.recv().await {
          match (frame.cid, frame.mid) {
            // Connection start
            (10, 10) => {
              info!("Sending [StartOk]");
              let client_properties = HashMap::from([
                ("product".to_string(), LongStr(PRODUCT.to_string())),
                ("platform".to_string(), LongStr(PLATFORM.to_string())),
                ("copyright".to_string(), LongStr(COPYRIGHT.to_string())),
                ("information".to_string(), LongStr(INFORMATION.to_string()))
              ]);
              // todo: add const for default channel or separate struct
              let start_ok = conn_methods::StartOk {
                properties: client_properties,
                mechanism: DEFAULT_AUTH_MECHANISM.to_string(),
                response: format!("\x00{}\x00{}", options.login.as_str(), options.password),
                locale: DEFAULT_LOCALE.to_string(),
              };

              let mut writer = writer.lock().await;
              writer.send_method(0, start_ok).await.unwrap();
            },
            // Tune method
            (10, 30) => {
              let tune_method: Frame2<conn_methods::Tune> = frame.into();
              let heartbeat_interval = tune_method.args.heartbeat;

              // todo: use values from conn options
              let tune_ok_method = conn_methods::TuneOk {
                chan_max: tune_method.args.chan_max,
                frame_max: tune_method.args.frame_max,
                heartbeat: tune_method.args.heartbeat,
              };
              info!("Sending [TuneOk]");

              let hb_writer = writer.clone();

              tokio::spawn(async move {
                loop {
                  {
                    let mut writer = hb_writer.lock().await;
                    let mut heartbeat_raw = vec![];
                    heartbeat_raw.write_byte(8).unwrap();
                    heartbeat_raw.write_short(0).unwrap();
                    Encode::write_int(&mut heartbeat_raw,0).unwrap();
                    heartbeat_raw.write_byte(0xCE).unwrap();
                    writer.write_binary(&mut heartbeat_raw).await.unwrap();
                    drop(writer);
                  }

                  tokio::time::sleep(tokio::time::Duration::from_secs(heartbeat_interval as u64)).await;
                }
              });


              let mut writer = writer.lock().await;
              writer.send_method(0, tune_ok_method).await.unwrap();
              info!("Sending [OpenMethod]");
              let open_method_frame = conn_methods::Open { vhost: options.vhost.clone(), ..conn_methods::Open::default() };
              writer.send_method(0, open_method_frame).await.unwrap();
            }
            // OpenOk method
            (10, 41) => {
              if tx.is_some() {
                let tx = tx.take().unwrap();
                tx.send(()).unwrap();
              }

              info!("Handshake completed");
              break;
            }
            _ => {
              println!("Unsupported frame");
            }
          }
        } else {
          // todo: review action
          panic!("Channel receiver channel");
        }
      }
    });

    rx.await.unwrap();
  }
}
