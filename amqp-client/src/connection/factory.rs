use tokio::net::TcpStream;
use super::{Connection, ConnectionOpts};
use crate::Result;

pub struct ConnectionFactory;

impl ConnectionFactory {
  pub async fn create(uri: &str) -> Result<Connection> {
    let options: ConnectionOpts = uri.into();
    let stream = TcpStream::connect(format!("{}:{}", options.host, options.port)).await?;

    let mut connection = Connection::new(stream, options);
    connection.connect().await?;

    Ok(connection)
  }
}
