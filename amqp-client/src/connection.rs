use std::net::ToSocketAddrs;
use crate::protocol::connection::AmqConnection;

pub struct Connection {
  raw: AmqConnection
}

impl Connection {
  pub fn new(host: String, port: u16, login: String, password: String, vhost: String) -> Self {
    Connection {
      raw: AmqConnection::new(
        host,
        port,
        login,
        password,
        vhost
      ),
    }
  }
}

pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String
}
