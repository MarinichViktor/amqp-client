use url::Url;

pub struct ConnectionArgs {
  pub address: ConnectionAddress,
  pub max_channels: i16,
  pub max_frame_size: i32,
  pub heartbeat_interval: i16,
}

impl ConnectionArgs {
  pub fn new(uri: &str) -> Self {
    Self {
      address: ConnectionAddress::from(uri),
      max_channels: 1024,
      max_frame_size: 128*1024,
      heartbeat_interval: 60
    }
  }
}


#[derive(Clone)]
pub struct ConnectionAddress {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String,
  pub vhost: String,
}

impl From<&str> for ConnectionAddress {
  fn from(uri: &str) -> Self {
    let url = Url::parse(uri).unwrap();
    let host = if url.has_host() {
      url.host().unwrap().to_string()
    } else {
      String::from("localhost")
    };
    let port = url.port().unwrap_or_else(|| 5672);
    let (login, password) = if url.has_authority() {
      (url.username().to_string(), url.password().unwrap().to_string())
    } else {
      panic!("Provide username and password in the connection url");
    };

    Self {
      host,
      port,
      login,
      password,
      vhost: url.path()[1..].into(),
    }
  }
}
