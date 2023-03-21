use url::Url;

#[derive(Clone)]
pub struct ConnectionOpts {
  pub host: String,
  pub port: u16,
  pub login: String,
  pub password: String,
  pub vhost: String,
}

impl From<&str> for ConnectionOpts {
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
