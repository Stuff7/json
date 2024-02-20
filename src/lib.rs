mod block;
mod parser;
mod state;
mod traits;

pub use block::JsonEntry;
pub use traits::*;

use block::JsonEntryIter;
use state::State;
use std::{
  isize,
  num::{ParseFloatError, ParseIntError},
  ops::Deref,
  str::Utf8Error,
  string::FromUtf8Error,
  usize,
};
use thiserror::Error;

#[derive(Debug)]
pub enum JsonValue<'a, 'b> {
  Null,
  Object(JsonEntryIter<'a, 'b>),
  Array(JsonIter<'a, 'b>),
  Integer(isize),
  Double(f64),
  String(Box<str>),
  Boolean(bool),
}

#[derive(Debug)]
pub struct JsonIter<'a, 'b> {
  state: &'a mut State<'b>,
  flags: u8,
  err: bool,
}

impl<'a, 'b> JsonIter<'a, 'b> {
  fn new(state: &'a mut State<'b>) -> Self {
    // Skip UTF-8 BOM
    if state.bytes[..3] == [0xEF, 0xBB, 0xBF] {
      state.bytes = &state.bytes[..3];
    }

    Self { state, err: false, flags: 0 }
  }

  fn from_state(state: &'a mut State<'b>) -> Self {
    state.next();
    Self {
      state,
      err: false,
      flags: IN_ARRAY,
    }
  }
}

impl<'a, 'b> Deref for JsonIter<'a, 'b> {
  type Target = State<'b>;

  fn deref(&self) -> &Self::Target {
    self.state
  }
}

impl<'a, 'b> LendingIterator for JsonIter<'a, 'b> {
  type Item<'this> = JsonResult<(JsonValue<'this, 'b>, usize, usize)> where Self: 'this;

  fn next(&mut self) -> Option<Self::Item<'_>> {
    let opt = self.state.peek(self.err, &mut self.flags).map(|b| {
      let mut ln = self.state.ln;
      let mut col = self.state.col;
      let res = {
        let b = b?;
        self.flags |= NEED_COMMA;
        match b {
          b'{' => Ok(JsonValue::Object(JsonEntryIter::from_state(self.state))),
          b'[' => Ok(JsonValue::Array(JsonIter::from_state(self.state))),
          _ => {
            let r = parser::parse_value(self.state, b, self.flags);
            ln = self.state.ln;
            col = self.state.col;
            r
          }
        }
      };
      self.err = res.is_err();
      res.map(|r| (r, ln, col))
    });

    if opt.is_none() {
      self.flags |= FINISHED;
    }

    opt
  }
}

impl<'a, 'b> Drop for JsonIter<'a, 'b> {
  fn drop(&mut self) {
    if self.flags & FINISHED == 0 && self.flags & IN_ARRAY != 0 {
      block::skip_block(self.state, b']');
    }
  }
}

#[derive(Debug, Error)]
pub enum JsonError {
  #[error("JSON doesn't match type")]
  NoMatch,
  #[error("{0}:{1}: Expected type {2:?}, found type {3:?}")]
  TypeMismatch(usize, usize, &'static str, &'static str),
  #[error("{0}:{1}: Unexpected EOF")]
  Eof(usize, usize),
  #[error("{0}:{1}: Unexpected escape sequence 0x{2:02X}")]
  Escape(usize, usize, u8),
  #[error("{0}:{1}: Expected digit after unicode sequence")]
  UnicodeLength(usize, usize),
  #[error(transparent)]
  ParseInt(#[from] ParseIntError),
  #[error(transparent)]
  ParseFloat(#[from] ParseFloatError),
  #[error(transparent)]
  Utf8(#[from] Utf8Error),
  #[error(transparent)]
  FromUtf8(#[from] FromUtf8Error),
  #[error("{0}:{1}: Not a unicode char '{2}'")]
  NotUnicode(usize, usize, u32),
  #[error("{0}:{1}: Expected `,` before `{2}`")]
  Comma(usize, usize, char),
  #[error("{0}:{1}: Expected `:` before `{2}`")]
  Colon(usize, usize, char),
  #[error("{0}:{1}: Expected `{2}`")]
  Expected(usize, usize, char),
  #[error("{0}:{1}: Unexpected `{2}`")]
  Unexpected(usize, usize, char),
}

pub type JsonResult<T> = Result<T, JsonError>;

const NEED_COLON: u8 = 1 << 1;
const IN_OBJECT: u8 = 1 << 2;
const NEED_KEY: u8 = 1 << 3;
const IN_ARRAY: u8 = 1 << 4;
const NEED_COMMA: u8 = 1 << 5;
const FINISHED: u8 = 1 << 6;
