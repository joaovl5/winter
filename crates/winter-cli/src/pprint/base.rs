use std::fmt::Display;

use console::StyledObject;

pub trait PrettyPrintable {
  type Printable: Display;
  fn pprint_str(&self) -> StyledObject<Self::Printable>;
  fn pprint(&self) {
    println!("{}", self.pprint_str());
  }
}

impl<T: Clone + Display> PrettyPrintable for StyledObject<T> {
  type Printable = T;

  fn pprint_str(&self) -> StyledObject<Self::Printable> {
    self.clone()
  }

  fn pprint(&self) {}
}
