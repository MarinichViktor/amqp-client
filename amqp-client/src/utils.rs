use std::sync::atomic::{AtomicI16, Ordering};

pub struct IdAllocator {
  prev_id: AtomicI16
}

impl IdAllocator {
  pub fn new() -> Self {
    Self {
      prev_id: AtomicI16::new(1)
    }
  }

  pub fn allocate(&mut self) -> i16 {
    self.prev_id.fetch_add(1, Ordering::Relaxed)
  }
}
