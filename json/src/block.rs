use std::ops::Deref;

use crate::{parser, JsonError, JsonIter, JsonResult, JsonValue, LendingIterator, State, FINISHED, IN_OBJECT, NEED_COLON, NEED_COMMA, NEED_KEY};

#[derive(Debug)]
pub struct JsonEntry<'a, 'b>(pub &'a str, pub JsonValue<'a, 'b>);

#[derive(Debug)]
pub struct JsonEntryIter<'a, 'b> {
  state: &'a mut State<'b>,
  flags: u8,
  err: bool,
}

impl<'a, 'b> JsonEntryIter<'a, 'b> {
  pub(super) fn from_state(state: &'a mut State<'b>) -> Self {
    state.next();
    Self {
      state,
      err: false,
      flags: NEED_KEY | IN_OBJECT,
    }
  }
}

impl<'a, 'b> Deref for JsonEntryIter<'a, 'b> {
  type Target = State<'a>;

  fn deref(&self) -> &Self::Target {
    self.state
  }
}

impl<'a, 'b> LendingIterator for JsonEntryIter<'a, 'b> {
  type Item<'this> = JsonResult<(JsonEntry<'this, 'b>, usize, usize)> where Self: 'this;

  fn next(&mut self) -> Option<Self::Item<'_>> {
    let mut entry = JsonEntry("", JsonValue::Null);
    let mut ln = self.state.ln;
    let mut col = self.state.col;

    let opt = self.state.peek(self.err, &mut self.flags).map(|b| {
      let res = {
        let mut b = b?;

        if self.flags & NEED_KEY != 0 {
          if b == b'"' {
            // This is safe it's just a str view into state.bytes (&[u8]) which is read-only
            // and never touched (state mutations below won't affect it) the other option is to
            // make key a Box<str> but this can affect performance in large json files with many keys
            entry.0 = unsafe { &*(parser::parse_string(self.state)? as *const _) };
            self.flags |= NEED_COLON;
            self.flags &= !NEED_KEY;
            b = self
              .state
              .peek(self.err, &mut self.flags)
              .ok_or(JsonError::Eof(self.state.ln, self.state.col))??;
          } else {
            return Err(JsonError::Unexpected(self.state.ln, self.state.col, b as char));
          }
        }

        if self.flags & NEED_COLON != 0 {
          if b == b':' {
            self.state.next().ok_or(JsonError::Eof(self.state.ln, self.state.col))?;
            b = self
              .state
              .peek(self.err, &mut self.flags)
              .ok_or(JsonError::Eof(self.state.ln, self.state.col))??;
            self.flags &= !NEED_COLON;
          } else {
            return Err(JsonError::Colon(self.state.ln, self.state.col, b as char));
          }
        }

        self.flags |= NEED_KEY | NEED_COMMA;
        let res = match b {
          b'{' => JsonValue::Object(JsonEntryIter::from_state(self.state)),
          b'[' => JsonValue::Array(JsonIter::from_state(self.state)),
          _ => {
            let r = parser::parse_value(self.state, b, self.flags)?;
            ln = self.state.ln;
            col = self.state.col;
            r
          }
        };

        entry.1 = res;
        Ok(entry)
      };
      self.err = res.is_err();
      res.map(|entry| (entry, ln, col))
    });

    if opt.is_none() {
      self.flags |= FINISHED;
    }

    opt
  }
}

impl<'a, 'b> Drop for JsonEntryIter<'a, 'b> {
  fn drop(&mut self) {
    if self.flags & FINISHED == 0 {
      skip_block(self.state, b'}');
    }
  }
}

pub fn skip_block(state: &mut State, closing_block: u8) {
  const ESCAPED: u8 = 1 << 1;
  const IN_STRING: u8 = 1 << 2;

  let mut flags = 0;
  let mut curly = 0;
  let mut square = 0;
  let mut quote = 0;

  for b in state.by_ref() {
    if (curly | square | quote) == 0 && b == closing_block {
      break;
    }

    if flags & IN_STRING != 0 {
      if b == b'"' && flags & ESCAPED == 0 {
        flags &= !IN_STRING;
        quote -= 1;
      } else if b == b'\\' {
        flags |= ESCAPED;
      } else if b != b'\\' {
        flags &= !ESCAPED;
      }
      continue;
    }

    match b {
      b'"' => {
        flags |= IN_STRING;
        quote += 1;
      }
      b'{' => curly += 1,
      b'}' => curly -= 1,
      b'[' => square += 1,
      b']' => square -= 1,
      _ => (),
    }
  }
}
