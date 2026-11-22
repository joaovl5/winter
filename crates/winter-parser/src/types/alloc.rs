use bumpalo::Bump;

#[derive(Debug, Default)]
pub struct WinterAlloc {
  pub arena: Bump,
}

impl WinterAlloc {
  pub fn new() -> Self {
    Self { arena: Bump::new() }
  }
}
