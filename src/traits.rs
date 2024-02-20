use crate::{JsonError, JsonIter, JsonResult, JsonValue, State};
use std::any::type_name;

pub trait LendingIterator {
  type Item<'this>
  where
    Self: 'this;

  fn next(&mut self) -> Option<Self::Item<'_>>;
}

impl<'a, 'b> JsonValue<'a, 'b> {
  pub fn name(&self) -> &'static str {
    match self {
      Self::Null => "null",
      Self::Object(_) => "object",
      Self::Array(_) => "array",
      Self::Integer(_) => "integer",
      Self::Double(_) => "double",
      Self::String(_) => "string",
      Self::Boolean(_) => "boolean",
    }
  }

  pub fn string(self, ln: usize, col: usize) -> JsonResult<Box<str>> {
    let JsonValue::String(s) = self else {
      return Err(JsonError::TypeMismatch(ln, col, "string", self.name()));
    };
    Ok(s)
  }

  pub fn int(self, ln: usize, col: usize) -> JsonResult<isize> {
    let JsonValue::Integer(s) = self else {
      return Err(JsonError::TypeMismatch(ln, col, "integer", self.name()));
    };
    Ok(s)
  }

  pub fn double(self, ln: usize, col: usize) -> JsonResult<f64> {
    let JsonValue::Double(s) = self else {
      return Err(JsonError::TypeMismatch(ln, col, "double", self.name()));
    };
    Ok(s)
  }

  pub fn bool(self, ln: usize, col: usize) -> JsonResult<bool> {
    let JsonValue::Boolean(s) = self else {
      return Err(JsonError::TypeMismatch(ln, col, "boolean", self.name()));
    };
    Ok(s)
  }
}

pub trait Json {
  fn json<T: JsonParser>(&self) -> JsonResult<T>;
}

impl Json for &[u8] {
  fn json<T: JsonParser>(&self) -> JsonResult<T> {
    T::from_json(self)
  }
}

impl Json for Vec<u8> {
  fn json<T: JsonParser>(&self) -> JsonResult<T> {
    T::from_json(self)
  }
}

pub trait JsonParser: Sized {
  fn parse_json(json: JsonValue) -> JsonResult<Self>;
  fn from_json(bytes: &[u8]) -> JsonResult<Self> {
    let mut state = State::new(bytes);
    let mut json = JsonIter::new(&mut state);
    let mut res = Err(JsonError::NoMatch);

    while let Some(value) = json.next() {
      res = Self::parse_json(value?.0);
      if res.is_ok() {
        return res;
      }
    }

    res
  }
}

impl<T: JsonParser> JsonParser for Vec<T> {
  fn parse_json(json: JsonValue) -> JsonResult<Self> {
    let JsonValue::Array(mut arr) = json else {
      return Err(JsonError::NoMatch);
    };

    let mut ret = Vec::new();
    while let Some(v) = arr.next() {
      let (v, ln, col) = v?;
      let name = v.name();
      let t = T::parse_json(v);
      match t {
        Ok(v) => ret.push(v),
        Err(e) => {
          if matches!(e, JsonError::NoMatch) {
            return Err(JsonError::TypeMismatch(ln, col, type_name::<T>(), name));
          }
          return Err(e);
        }
      }
    }

    Ok(ret)
  }
}

impl<T: JsonParser> JsonParser for Option<T> {
  fn parse_json(json: JsonValue) -> JsonResult<Self> {
    if let JsonValue::Null = json {
      return Ok(None);
    }

    T::parse_json(json).map(Some)
  }
}

macro_rules! json_primitive {
  ($type: ident, $self: ty) => {
    impl JsonParser for $self {
      fn parse_json(json: JsonValue) -> JsonResult<Self> {
        let JsonValue::$type(s) = json else {
          return Err(JsonError::NoMatch);
        };
        Ok(s as $self)
      }
    }
  };
}

json_primitive!(Integer, isize);
json_primitive!(Integer, usize);
json_primitive!(Integer, i128);
json_primitive!(Integer, u128);
json_primitive!(Integer, i64);
json_primitive!(Integer, u64);
json_primitive!(Integer, i32);
json_primitive!(Integer, u32);
json_primitive!(Integer, i16);
json_primitive!(Integer, u16);
json_primitive!(Integer, i8);
json_primitive!(Integer, u8);

json_primitive!(Double, f64);
json_primitive!(Double, f32);

json_primitive!(String, Box<str>);

json_primitive!(Boolean, bool);
