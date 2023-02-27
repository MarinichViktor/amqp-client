pub use crate::protocol::connection::AmqConnection as Connection;
pub use crate::protocol::channel::AmqChannel as Channel;
// use crate::channel::Channel;
// use crate::protocol::connection::AmqConnection;
// use crate::{Result};
//
// pub struct Connection {
//   raw: AmqConnection
// }
//
// impl Connection {
//   pub fn new(host: String, port: u16, login: String, password: String, vhost: String) -> Self {
//     Connection {
//       raw: AmqConnection::new(
//         host,
//         port,
//         login,
//         password,
//         vhost
//       ),
//     }
//   }
//
//   pub fn open(&mut self) -> Result<()> {
//     self.raw.connect()?;
//     Ok(())
//   }
//
//   pub fn create_channel(&mut self) -> Result<Channel> {
//     let raw_channel = self.raw.create_channel()?;
//     Ok(Channel::new(raw_channel))
//   }
// }
