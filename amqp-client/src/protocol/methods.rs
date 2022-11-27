use std::collections::HashMap;
use amqp_protocol::types::{Property};

pub mod connection;
pub mod channel;

pub type PropTable = HashMap<String, Property>;

// pub fn get_frame_id(method_frame: &MethodFrameArgs) -> (i16, i16) {
//   let mut class_id;
//   let mut method_id;
//   match method_frame {
//     MethodFrameArgs::Conn(conn_method) => {
//       class_id = 10;
//
//       match conn_method {
//         ConnMethodArgs::Start(_) => {
//           method_id = 10;
//         }
//         ConnMethodArgs::StartOk(_) => {
//           method_id = 11;
//         }
//       }
//     }
//   }
//
//   (class_id, method_id)
// }
