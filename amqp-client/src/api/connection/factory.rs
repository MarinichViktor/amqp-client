use tokio::net::TcpStream;
use crate::api::connection::options::ConnectionArgs;
use super::{Connection, ConnectionAddress};
use crate::Result;

pub struct ConnectionFactory;

impl ConnectionFactory {
  pub async fn create(uri: &str) -> Result<Connection> {
    let options = ConnectionArgs::new(uri);
    let stream = TcpStream::connect((options.address.host.clone(), options.address.port)).await?;
    let connection = Connection::open(stream, options).await?;
    Ok(connection)
  }
}
