// use amqp_macros::amqp_method;
// use crate::protocol::types::Table;
// use crate::exchange::{ExchangeDeclareOpts, ExchangeType};
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=40, m_id=10)]
// pub struct Declare {
//   #[short]
//   pub reserved1: i16,
//   #[short_str]
//   pub exchange: String,
//   #[short_str]
//   pub ty: String,
//   #[byte]
//   pub flags: u8,
//   #[prop_table]
//   pub table: Table,
// }
//
// const PASSIVE_MASK: u8 = 0b01;
// const DURABLE_MASK: u8 = 0b10;
// const AUTODELETE_MASK: u8 = 0b100;
// const INTERNAL_MASK: u8 = 0b1000;
// const NOWAIT_MASK: u8 = 0b10000;
//
// impl From<ExchangeDeclareOpts> for Declare {
//   fn from(options: ExchangeDeclareOpts) -> Self {
//     let ty = match options.ty {
//       ExchangeType::Direct => "direct",
//       ExchangeType::Fanout => "fanout"
//     };
//
//     let mut flags = 0;
//
//     if options.passive {
//       flags = flags & PASSIVE_MASK;
//     }
//
//     if options.durable {
//       flags = flags & DURABLE_MASK;
//     }
//
//     if options.auto_delete {
//       flags = flags & AUTODELETE_MASK;
//     }
//
//     if options.internal {
//       flags = flags & INTERNAL_MASK;
//     }
//
//     if options.no_wait {
//       flags = flags & NOWAIT_MASK;
//     }
//
//     Self {
//       reserved1: 0,
//       exchange: options.name,
//       ty: ty.to_string(),
//       flags,
//       table: options.props
//     }
//   }
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=40, m_id=11)]
// pub struct DeclareOk {
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=40, m_id=20)]
// pub struct Delete {
//   #[short_str]
//   pub reserved1: String,
//   #[short_str]
//   pub exchange: String,
//   #[byte]
//   pub if_unused: u8,
//   #[byte]
//   pub no_wait: u8,
// }
//
// #[derive(Debug, Default)]
// #[amqp_method(c_id=30, m_id=21)]
// pub struct DeleteOk {
// }
