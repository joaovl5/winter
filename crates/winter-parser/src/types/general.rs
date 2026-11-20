use std::fmt::Display;

use itertools::Itertools;

/// Span with `START` and `STOP` positions.
#[derive(Debug)]
pub struct Span(pub usize, pub usize);
#[derive(Debug)]
pub struct Spanned<T>(pub Span, pub T);

impl Display for Span {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "@{}:{}", self.0, self.1)
  }
}

impl<T: Display> Display for Spanned<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.0, self.1)
  }
}

#[derive(thiserror::Error, Debug)]
#[error("Error at {0}: {1}")]
pub struct SpannedError<E: std::error::Error + Send + Sync + 'static>(
  pub Span,
  #[source] pub E,
);

#[derive(Debug)]
pub enum WantedCandidates<T: Display> {
  ONE(T),
  ONE_OF(Vec<T>),
}

impl<T: Display> Display for WantedCandidates<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      | Self::ONE(x) => write!(f, "'{x}'"),
      | Self::ONE_OF(xs) => write!(
        f,
        "one of: {}",
        xs.iter().map(|x| format!("'{x}'")).join(", ")
      ),
    }
  }
}
