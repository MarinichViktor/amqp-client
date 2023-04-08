// use amqp_macros::amqp_method;
// use crate::protocol::types::{Byte, LongStr, ShortStr, Table};
//
// #[derive(Debug)]
// #[amqp_method(c_id=10, m_id=10)]
// pub struct Start {
//   #[byte]
//   pub ver_major: Byte,
//   #[byte]
//   pub ver_minor: Byte,
//   #[prop_table]
//   pub properties: Table,
//   #[long_str]
//   pub mechanisms: LongStr,
//   #[long_str]
//   pub locales: LongStr,
// }
//
// #[derive(Debug)]
// #[amqp_method(c_id=10, m_id=11)]
// pub struct StartOk {
//   #[prop_table]
//   pub properties: Table,
//   #[short_str]
//   pub mechanism: ShortStr,
//   #[long_str]
//   pub response: LongStr,
//   #[short_str]
//   pub locale: ShortStr,
// }
//
// #[derive(Debug)]
// #[amqp_method(c_id=10, m_id=30)]
// pub struct Tune {
//   #[short]
//   pub chan_max: i16,
//   #[int]
//   pub frame_max: i32,
//   #[short]
//   pub heartbeat: i16,
// }
//
// #[derive(Debug)]
// #[amqp_method(c_id=10, m_id=31)]
// pub struct TuneOk {
//   #[short]
//   pub chan_max: i16,
//   #[int]
//   pub frame_max: i32,
//   #[short]
//   pub heartbeat: i16,
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=10, m_id=40)]
// pub struct Open {
//   #[short_str]
//   pub vhost: ShortStr,
//   #[short_str]
//   pub reserved1: ShortStr,
//   #[byte]
//   pub reserved2: u8,
// }
//
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=10, m_id=41)]
// pub struct OpenOk {
//   #[short_str]
//   pub reserved1: ShortStr,
// }
