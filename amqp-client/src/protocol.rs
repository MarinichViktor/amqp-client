pub mod frame;

pub use crate::protocol::types::{AmqpMethodArgs, PropTable};

pub mod basic;
pub (crate) mod reader;
pub (crate) mod writer;
pub mod frame2;
pub mod enc;
pub mod dec;
pub mod types;
use paste::paste;

#[macro_export]
macro_rules! define_amqp_classes {
  (
    $(
      $class:ident($class_id:literal) {
        $(
          $method:ident($method_id:literal) {
            $($field:ident : $type:ty),+
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
            $(pub(crate) $field : $type),+
          }

          impl [<$class $method>]  {
            pub fn from_raw_repr(mut buf: &[u8]) -> Self {
              // discard class and method id
              buf.read_short().unwrap();
              buf.read_short().unwrap();
              $(
                let $field = buf.[<read_ $type:lower>]().unwrap();
              )+
              // $(let $field = 1_u16;)+
              Self {
                $($field),+
              }
            }

            pub fn to_raw_repr(self) -> Vec<u8> {
              let mut buf = vec![];
              buf.write_short($class_id).unwrap();
              buf.write_short($method_id).unwrap();
              $(
                let $field = buf.[<write_ $type:lower >](self.$field).unwrap();
              )+
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
      pub enum Frame {
        $(
          $([<$class $method>]([<$class $method>])),+
        )+,
        ContentHeader,
        ContentBody,
        Heartbeat
      }

      impl Frame {
        pub fn method(channel: Short, class_id: Short, method_id: Short, body: &[u8]) -> Self {
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
            _ => {
              todo!("implement")
            }
          }
        }
      }
    }
  }
}
