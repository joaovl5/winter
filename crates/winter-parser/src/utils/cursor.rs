use std::alloc::{Allocator, Global};
use std::collections::BTreeMap;
use std::num::NonZero;

use itertools::{PeekNth, peek_nth};

use crate::{ok, ok_none, ok_some};

/// Manager of an internal iterator with capabilities for potentially
/// moving back in time at select times when explicitly desired, otherwise
/// no allocations are made.
#[derive(Debug, Clone)]
pub struct Cursor<T, I: Iterator<Item = T>> {
  pub pos:   usize,
  front_pos: usize,
  iter:      PeekNth<I>,
  pos_stack: Vec<usize>,
  cache:     BTreeMap<usize, T>,
}

#[derive(thiserror::Error, Debug)]
pub enum CursorError {
  #[error("Expected value in cache for position {0}, but None found.")]
  ExpectedCacheKey(usize),

  #[error("Tried to rollback but no position was stored for going back to!")]
  NothingToRollback,

  #[error("Tried to commit but no checkpoint was stored!")]
  NothingToCommit,
}

pub type CursorRes<T> = Result<T, CursorError>;

impl<T: Copy, I: Iterator<Item = T>> Cursor<T, I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter:      peek_nth(iter),
      pos:       0,
      front_pos: 0,
      pos_stack: Vec::new(),
      cache:     BTreeMap::new(),
    }
  }

  pub fn peek(&mut self) -> CursorRes<Option<T>> {
    self.peek_nth(0)
  }

  pub fn peek_nth(&mut self, n: usize) -> CursorRes<Option<T>> {
    let peek_pos = self.pos + n;
    if peek_pos < self.front_pos {
      let cached = self
        .cache
        .get(&peek_pos)
        .copied()
        .ok_or(CursorError::ExpectedCacheKey(peek_pos))?;
      ok_some!(cached);
    }

    let value = self
      .iter
      .peek_nth(peek_pos - self.front_pos)
      .copied();
    if self.is_recording()
      && let Some(inner) = value
    {
      self.cache.insert(peek_pos, inner);
    }
    ok!(value);
  }

  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> CursorRes<Option<T>> {
    if self.pos < self.front_pos {
      let cached = self
        .cache
        .get(&self.pos)
        .copied()
        .ok_or(CursorError::ExpectedCacheKey(self.pos))?;
      self.pos += 1;
      ok_some!(cached);
    }

    let Some(value) = self.iter.next() else {
      ok_none!();
    };

    if self.is_recording() {
      self.cache.insert(self.pos, value);
    }
    self.pos += 1;
    self.front_pos += 1;
    ok_some!(value);
  }

  /// Advances by up to `n` positions, stops at EOF.
  pub fn advance_by(&mut self, n: NonZero<usize>) -> CursorRes<Option<T>> {
    let mut last: Option<T> = None;
    for i in 1..=n.into() {
      last = self.next()?;
    }
    ok!(last);
  }

  /// Whether we should be "recording" elements that come by. This is only
  /// true while at least one checkpoint is active.
  pub fn is_recording(&self) -> bool {
    !self.pos_stack.is_empty()
  }

  pub fn pos_checkpoint(&mut self) {
    self.pos_stack.push(self.pos);
  }

  pub fn pos_rollback(&mut self) -> CursorRes<()> {
    self.pos = self
      .pos_stack
      .pop()
      .ok_or(CursorError::NothingToRollback)?;

    self.prune_cache();
    Ok(())
  }

  pub fn pos_commit(&mut self) -> CursorRes<()> {
    self
      .pos_stack
      .pop()
      .ok_or(CursorError::NothingToCommit)?;

    self.prune_cache();
    Ok(())
  }

  fn prune_cache(&mut self) {
    let keep_from = self
      .pos_stack
      .first()
      .copied()
      .unwrap_or(self.pos);

    self.cache = self.cache.split_off(&keep_from);
  }
}
