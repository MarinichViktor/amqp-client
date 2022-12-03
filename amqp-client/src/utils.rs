use std::sync::Mutex;

// todo: refactor this
pub struct IdAllocator {
  prev_id: Mutex<i16>
}

impl IdAllocator {
  pub fn new() -> Self {
    Self { prev_id: Mutex::new(0) }
  }

  pub fn allocate(&mut self) -> i16 {
    let mut prev_id = self.prev_id.lock().unwrap();
    *prev_id += 1;
    *prev_id
  }
}
