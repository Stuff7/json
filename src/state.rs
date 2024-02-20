use crate::{JsonError, JsonResult, IN_ARRAY, IN_OBJECT, NEED_COMMA};

#[derive(Debug)]
pub struct State<'a> {
  pub(super) bytes: &'a [u8],
  pub(super) ptr: usize,
  pub(super) str_buf: Vec<u8>,
  pub ln: usize,
  pub col: usize,
}

impl<'a> State<'a> {
  pub(super) fn new(bytes: &'a [u8]) -> Self {
    Self {
      bytes,
      ptr: 0,
      str_buf: Vec::with_capacity(256),
      ln: 1,
      col: 1,
    }
  }

  pub(super) fn peek(&mut self, err: bool, flags: &mut u8) -> Option<JsonResult<u8>> {
    if err {
      return None;
    }

    let mut b;
    loop {
      b = self.bytes.get(self.ptr).copied()?;
      if b.is_ascii_whitespace() {
        self.next()?;
        continue;
      }

      if (*flags & IN_OBJECT != 0 && b == b'}') || (*flags & IN_ARRAY != 0 && b == b']') {
        self.next()?;
        return None;
      }

      if *flags & NEED_COMMA != 0 {
        if b == b',' {
          *flags &= !NEED_COMMA;
          self.next()?;
          continue;
        }

        return Some(Err(JsonError::Comma(self.ln, self.col, b as char)));
      }

      return Some(Ok(b));
    }
  }
}

impl<'a> Iterator for State<'a> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.ptr >= self.bytes.len() {
      return None;
    }

    let b = self.bytes[self.ptr];
    if b == 10 {
      self.ln += 1;
      self.col = 1;
    } else {
      self.col += 1;
    }

    self.ptr += 1;
    Some(b)
  }
}
