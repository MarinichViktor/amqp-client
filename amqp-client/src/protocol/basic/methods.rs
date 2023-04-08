// use amqp_macros::amqp_method;
// use crate::protocol::types::Table;
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=60, m_id=20)]
// pub struct Consume {
//   #[short]
//   pub reserved1: i16,
//   #[short_str]
//   pub queue: String,
//   #[short_str]
//   pub tag: String,
//   // todo: add params for no-local, noack, exclusive,no-wait
//   #[byte]
//   pub flags: u8,
//   #[prop_table]
//   pub table: Table,
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=60, m_id=21)]
// pub struct ConsumeOk {
//   #[short_str]
//   pub tag: String,
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=60, m_id=60)]
// pub struct Deliver {
//   #[short_str]
//   pub consumer_tag: String,
//   #[long]
//   pub deliver_tag: i64,
//   #[byte]
//   pub redelivered: u8,
//   #[short_str]
//   pub exchange: String,
//   #[short_str]
//   pub routing_key: String,
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=60, m_id=40)]
// pub struct Publish {
//   #[short]
//   pub reserved1: i16,
//   #[short_str]
//   pub exchange: String,
//   #[short_str]
//   pub routing_key: String,
//   // todo: add params
//   #[byte]
//   pub flags: u8,
// }
