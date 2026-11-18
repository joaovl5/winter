use std::fmt::Display;

use itertools::Itertools;

/// Span with `START` and `STOP` positions.
pub struct Span(pub usize, pub usize);
pub struct Spanned<T>(pub Span, pub T);

#[derive(thiserror::Error)]
pub struct SpannedError<E: std::error::Error>(pub Span, #[source] pub E);

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
