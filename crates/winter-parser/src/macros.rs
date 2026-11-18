/// Just does `return Ok(<value>)`
#[macro_export]
macro_rules! ok {
  ($x:expr) => {
    return Ok($x)
  };
}

/// Just does `return Err(<value>)`
#[macro_export]
macro_rules! err {
  ($x:expr) => {
    return Err($x)
  };
}

/// Just does `return Ok(None)`.
#[macro_export]
macro_rules! ok_none {
  () => {
    return Ok(None)
  };
}

/// Just does `return Ok(Some(<value>))`.
#[macro_export]
macro_rules! ok_some {
  ($x:expr) => {
    return Ok(Some($x))
  };
}

/// Encapsulates a `T` by making a [`crate::types::Spanned`] instance.
///
/// - `span_this!(x, obj)` uses `x` for both start and end of the span
/// - `span_this!(x, y, obj)` uses `x` and `y` respectively
#[macro_export]
macro_rules! span_this {
  ($pos_start:expr, $x:expr) => {
    $crate::types::Spanned($crate::types::Span($pos_start, $pos_start), $x)
  };
  ($pos_start:expr, $pos_end:expr, $x:expr) => {
    $crate::types::Spanned($crate::types::Span($pos_start, $pos_end), $x)
  };
}

/// It's like `span_this!` but for [`SpannedError`], and also prefixes a return.
#[macro_export]
macro_rules! span_err {
  ($pos_start:expr, $err:expr) => {
    return Err($crate::types::SpannedError(
      $crate::types::Span($pos_start, $pos_start),
      $err,
    ))
  };
  ($pos_start:expr, $pos_end:expr, $err:expr) => {
    return Err($crate::types::SpannedError(
      $crate::types::Span($pos_start, $pos_end),
      $err,
    ))
  };
}
