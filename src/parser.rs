use std::str;

use crate::JsonError;
use crate::JsonResult;
use crate::JsonValue;
use crate::State;
use crate::IN_ARRAY;
use crate::IN_OBJECT;

pub fn parse_value<'a, 'b>(state: &mut State, b: u8, parent_flags: u8) -> JsonResult<JsonValue<'a, 'b>> {
  match b {
    b'"' => parse_string(state).map(|s| JsonValue::String(s.into())),
    b if b == b'-' || b.is_ascii_digit() => parse_number(state, parent_flags),
    b'n' | b'f' | b't' => parse_keyword(state),
    _ => Err(JsonError::Comma(state.ln, state.col, b as char)),
  }
}

pub fn parse_string<'a>(state: &'a mut State) -> JsonResult<&'a str> {
  if state.next().is_some_and(|b| b != b'"') {
    return Err(JsonError::Expected(state.ln, state.col, '"'));
  }

  const ESCAPED: u8 = 1 << 1;
  const BUFFER: u8 = 1 << 2;

  let start = state.ptr;
  let end;
  let mut flags = 0;
  let mut extra_byte: Option<u8> = None;

  loop {
    let opt = extra_byte.take().or_else(|| state.next());

    let Some(b) = opt else {
      return Err(JsonError::Eof(state.ln, state.col));
    };

    if b == b'\n' {
      return Err(JsonError::Escape(state.ln, state.col, b));
    }

    if flags & ESCAPED == 0 {
      match b {
        b'\\' => {
          if flags & BUFFER == 0 {
            state.str_buf.truncate(0);
            state.str_buf.extend_from_slice(&state.bytes[start..state.ptr - 1]);
          }
          flags |= ESCAPED | BUFFER;
        }
        b'"' => {
          end = state.ptr - 1;
          break;
        }
        _ => {
          if flags & BUFFER != 0 {
            state.str_buf.push(b);
          }
        }
      }
      continue;
    }

    flags &= !ESCAPED;
    let c = match b {
      b'b' => b'\x08',
      b't' => b'\x09',
      b'n' => b'\x0A',
      b'f' => b'\x0C',
      b'r' => b'\x0D',
      b'u' => {
        let mut u = 0;
        for i in (0..4).rev() {
          let Some(hex) = state.next().and_then(hex_value) else {
            return Err(JsonError::UnicodeLength(state.ln, state.col));
          };
          u += hex << (i << 2);
        }

        extra_byte = state.next();
        if let Some(fifth) = extra_byte {
          if let Some(fifth) = hex_value(fifth) {
            u <<= 4;
            u += fifth;
          }
        }

        if u <= 0x7F {
          state.str_buf.push(u as u8);
        } else if u <= 0x7FF {
          state.str_buf.push((0xC0 | (u >> 6)) as u8);
          state.str_buf.push((0x80 | (u & 0x3F)) as u8);
        } else if u <= 0xFFFF {
          state.str_buf.push((0xE0 | (u >> 12)) as u8);
          state.str_buf.push((0x80 | ((u >> 6) & 0x3F)) as u8);
          state.str_buf.push((0x80 | (u & 0x3F)) as u8);
        } else if u <= 0x1869F {
          state.str_buf.push((0xF0 | (u >> 18)) as u8);
          state.str_buf.push((0x80 | ((u >> 12) & 0x3F)) as u8);
          state.str_buf.push((0x80 | ((u >> 6) & 0x3F)) as u8);
          state.str_buf.push((0x80 | (u & 0x3F)) as u8);
          extra_byte = None;
        } else {
          return Err(JsonError::NotUnicode(state.ln, state.col, u));
        }

        continue;
      }
      _ => b,
    };

    state.str_buf.push(c);
  }

  Ok(str::from_utf8(if flags & BUFFER == 0 {
    &state.bytes[start..end]
  } else {
    &state.str_buf
  })?)
}

fn parse_number<'a, 'b>(state: &mut State, parent_flags: u8) -> JsonResult<JsonValue<'a, 'b>> {
  const E_NOTATION: u8 = 1 << 1;
  const DBL: u8 = 1 << 2;

  let mut flags = 0;
  let start = state.ptr;
  let mut end = 0;

  while let Some(b) = state.next() {
    match b {
      b'-' => {
        if state.bytes[start..state.ptr - 1].last().is_some_and(|b| *b != b'e') {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
      }
      b'e' => {
        if flags & E_NOTATION != 0 {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
        flags |= E_NOTATION;
      }
      b'.' => {
        if flags & DBL != 0 {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
        flags |= DBL;
      }
      _ => {
        if let Some(b) = state.bytes.get(state.ptr) {
          if b == &b',' || b.is_ascii_whitespace() || (parent_flags & IN_ARRAY != 0 && b == &b'[') || (parent_flags & IN_OBJECT != 0 && b == &b'{') {
            end = state.ptr;
            break;
          }
        }

        if !b.is_ascii_digit() {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
      }
    }
  }

  let s = str::from_utf8(&state.bytes[start..end])?;
  Ok(if flags & (DBL | E_NOTATION) == 0 {
    JsonValue::Integer(s.parse()?)
  } else {
    JsonValue::Double(s.parse()?)
  })
}

fn parse_keyword<'a, 'b>(state: &mut State) -> JsonResult<JsonValue<'a, 'b>> {
  let Some(b) = state.next() else {
    return Err(JsonError::Eof(state.ln, state.col));
  };

  let v = match b {
    b'n' => {
      for c in b"ull" {
        if state.next().ok_or(JsonError::Eof(state.ln, state.col))? != *c {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
      }
      JsonValue::Null
    }
    b'f' => {
      for c in b"alse" {
        if state.next().ok_or(JsonError::Eof(state.ln, state.col))? != *c {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
      }
      JsonValue::Boolean(false)
    }
    b't' => {
      for c in b"rue" {
        if state.next().ok_or(JsonError::Eof(state.ln, state.col))? != *c {
          return Err(JsonError::Unexpected(state.ln, state.col, b as char));
        }
      }
      JsonValue::Boolean(true)
    }
    _ => return Err(JsonError::Unexpected(state.ln, state.col, b as char)),
  };

  Ok(v)
}

fn hex_value(c: u8) -> Option<u32> {
  (c as char).to_digit(16)
}
