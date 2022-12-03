use amqp_macros::amqp_method;

pub const CLASS_CHANNEL: i16 = 20;
pub const METHOD_OPEN: i16 = 10;
pub const METHOD_OPEN_OK: i16 = 11;

#[derive(Debug, Default)]
#[amqp_method(c_id=20, m_id=10)]
pub struct Open {
  #[short_str]
  pub reserved1: String,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=20, m_id=11)]
pub struct OpenOk {
  #[short_str]
  pub reserved1: String,
}

#[derive(Debug)]
#[amqp_method(c_id=20, m_id=20)]
pub struct Flow {
  #[byte]
  pub active: u8,
}

#[derive(Debug)]
#[amqp_method(c_id=20, m_id=21)]
pub struct FlowOk {
  #[byte]
  pub active: u8,
}

#[derive(Debug)]
#[amqp_method(c_id=20, m_id=40)]
pub struct Close {
  #[short]
  pub reply_code: i16,
  #[short_str]
  pub reply_text: String,
  #[short]
  pub class_id: i16,
  #[short]
  pub method_id: i16,
}

#[derive(Debug, Default)]
#[amqp_method(c_id=20, m_id=41)]
pub struct CloseOk {
}
