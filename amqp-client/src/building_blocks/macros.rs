use paste::paste;

#[macro_export]
macro_rules! unwrap_frame_variant {
  (
    $enum: expr, $variant:ident
  ) => {
    match $enum {
      Frame::$variant(payload) => payload,
      _ => panic!("Failed to unwrap variant")
    }

  }
}

#[macro_export]
macro_rules! generate_protocol_methods {
  (
    $(
      $class:ident($class_id:literal) {
        $(
          $method:ident($method_id:literal) {
            $($field:ident : $type:ty,)*
          }
        )+
      }
    )+
  ) => {
    $(
      $(
        paste! {
          #[derive(Debug)]
          pub struct [<$class $method>] {
            $(pub(crate) $field : $type,)*
          }

          impl [<$class $method>]  {
            pub fn from_raw_repr(mut buf: &[u8]) -> Self {
              // discard class and method id
              buf.read_short().unwrap();
              buf.read_short().unwrap();
              $(
                let $field = buf.[<read_ $type:lower>]().unwrap();
              )*
              // $(let $field = 1_u16;)+
              Self {
                $($field),*
              }
            }

            pub fn to_raw_repr(self) -> Vec<u8> {
              let mut buf = vec![];
              buf.write_short($class_id).unwrap();
              buf.write_short($method_id).unwrap();
              $(
                let $field = buf.[<write_ $type:lower >](self.$field).unwrap();
              )*
              buf
            }

            pub fn class_id(&self) -> Short {
              $class_id
            }

            pub fn method_id(&self) -> Short {
              $method_id
            }

            pub fn into_frame(self) -> Frame {
              Frame::[<$class $method>](self)
            }
          }
        }
      )+
    )+

    paste! {
      #[derive(Debug)]
      pub enum Frame {
        $(
          $(
            [<$class $method>]([<$class $method>]),
          )+
        )+
        ContentHeader(ContentHeader),
        ContentBody(ContentBody),
        Heartbeat
      }

      impl Frame {
        pub fn method(class_id: Short, method_id: Short, body: &[u8]) -> Self {
          return match class_id {
           $(
              $class_id => {
                match method_id {
                  $(
                    $method_id => {
                      Frame::[<$class $method>]([<$class $method>]::from_raw_repr(body))
                    }
                  ),+
                  _ => {
                    panic!("Unsupported method")
                  }
                }
              }
           ),+
           _ => {
             panic!("Unsupported class")
           }
          }
        }

        pub fn to_raw_repr(self) -> Vec<u8> {
          match self {
            $(
              $(
                Frame::[<$class $method>](payload) => {
                  payload.to_raw_repr()
                }
              )+
            )+,
            Frame::ContentHeader(header) => {
              header.to_raw_repr()
            },
            Frame::ContentBody(body) => {
              body.to_raw_repr()
            },
            Frame::Heartbeat => {
              vec![]
            }
          }
        }
      }
    }
  }
}

#[macro_export]
macro_rules! invoke_command_async {
  (
    $command_tx:expr,
    $payload:expr
  ) => {
    use tokio::sync::oneshot;
    let (ack_tx, ack_rx) = oneshot::channel::<()>();
    // todo: review
    $command_tx.send(($payload, ack_tx))?;
    ack_rx.await?;
  }
}

// todo: rewrite as function?
#[macro_export]
macro_rules! invoke_sync_method {
  (
    $channel:expr,
    $command_tx:expr,
    $outgoing_tx:expr,
    $payload:expr
  ) => {
    {
      let (responder_tx, responder_rx) = oneshot::channel::<Frame>();
      invoke_command_async!($command_tx, CommandPayload::RegisterResponder(($channel, responder_tx)));

      $outgoing_tx.send(($channel, $payload)).unwrap();
      responder_rx
    }
  }
}
