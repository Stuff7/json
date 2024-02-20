use json::{Json, JsonEntry, JsonError, JsonParser, JsonValue, LendingIterator};
use std::fmt::Debug;
use std::{fs, time::Instant};
use thiserror::Error;

fn main() -> Result<(), MainError> {
  let filename = std::env::args().nth(1).ok_or(MainError::Usage)?;
  let bytes = fs::read(&filename)?;
  let ellapsed = Instant::now();
  let sample: Vec<Option<MockData>> = bytes.json()?;
  let ellapsed = Instant::now() - ellapsed;
  println!("{:#?}", sample);
  println!("File: {filename} ({} bytes)", bytes.len());
  println!("Done in {ellapsed:?}");
  Ok(())
}

#[derive(Error)]
enum MainError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("Usage: json <file>")]
  Usage,
  #[error(transparent)]
  Json(#[from] JsonError),
}

impl Debug for MainError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

#[derive(Debug, Default)]
struct Preferences {
  theme: Box<str>,
  language: Box<str>,
}

#[derive(Debug, Default)]
struct UserDetails {
  email: Box<str>,
  phone_number: Box<str>,
  is_verified: bool,
  preferences: Preferences,
}

#[derive(Debug, Default)]
struct RelatedUser {
  username: Box<str>,
  user_id: u64,
  is_active: bool,
}

#[derive(Debug, Default)]
struct MockData {
  username: Box<str>,
  user_id: u64,
  is_active: bool,
  mock_data: Vec<Box<str>>,
  user_details: UserDetails,
  related_users: Vec<RelatedUser>,
  last_login: Box<str>,
  website: Box<str>,
  validation_regex: Box<str>,
  binary_data: Box<str>,
}

impl JsonParser for Preferences {
  fn parse_json(json: JsonValue) -> json::JsonResult<Self> {
    if let JsonValue::Object(mut obj) = json {
      let mut preferences = Preferences::default();
      let mut flags: u8 = 0b11;
      while let Some(res) = obj.next() {
        let res = res?;
        let (JsonEntry(key, value), ln, col) = res;
        match key {
          "theme" => {
            preferences.theme = value.string(ln, col)?;
            flags &= !0b10;
          }
          "language" => {
            preferences.language = value.string(ln, col)?;
            flags &= !0b01;
          }
          _ => (),
        }

        if flags == 0 {
          break;
        }
      }

      return Ok(preferences);
    }
    Err(json::JsonError::NoMatch)
  }
}

impl JsonParser for UserDetails {
  fn parse_json(json: JsonValue) -> json::JsonResult<Self> {
    if let JsonValue::Object(mut obj) = json {
      let mut userdetails = UserDetails::default();
      let mut flags: u8 = 0b1111;
      while let Some(res) = obj.next() {
        let res = res?;
        let (JsonEntry(key, value), ln, col) = res;
        match key {
          "email" => {
            userdetails.email = value.string(ln, col)?;
            flags &= !0b1000;
          }
          "phone_number" => {
            userdetails.phone_number = value.string(ln, col)?;
            flags &= !0b0100;
          }
          "is_verified" => {
            userdetails.is_verified = value.bool(ln, col)?;
            flags &= !0b0010;
          }
          "preferences" => {
            userdetails.preferences = Preferences::parse_json(value)?;
            flags &= !0b0001;
          }
          _ => (),
        }

        if flags == 0 {
          break;
        }
      }

      return Ok(userdetails);
    }

    Err(json::JsonError::NoMatch)
  }
}

impl JsonParser for RelatedUser {
  fn parse_json(json: JsonValue) -> json::JsonResult<Self> {
    if let JsonValue::Object(mut obj) = json {
      let mut relateduser = RelatedUser::default();
      let mut flags: u8 = 0b111;
      while let Some(res) = obj.next() {
        let res = res?;
        let (JsonEntry(key, value), ln, col) = res;
        match key {
          "username" => {
            relateduser.username = value.string(ln, col)?;
            flags &= !0b100;
          }
          "user_id" => {
            relateduser.user_id = value.int(ln, col)? as u64;
            flags &= !0b010;
          }
          "is_active" => {
            relateduser.is_active = value.bool(ln, col)?;
            flags &= !0b001;
          }
          _ => (),
        }

        if flags == 0 {
          break;
        }
      }

      return Ok(relateduser);
    }
    Err(json::JsonError::NoMatch)
  }
}

impl JsonParser for MockData {
  fn parse_json(json: JsonValue) -> json::JsonResult<Self> {
    if let JsonValue::Object(mut obj) = json {
      let mut mockdata = MockData::default();
      let mut flags: u16 = 0b1111111111;
      while let Some(res) = obj.next() {
        let res = res?;
        let (JsonEntry(key, value), ln, col) = res;
        match key {
          "username" => {
            mockdata.username = value.string(ln, col)?;
            flags &= !0b1000000000;
          }
          "user_id" => {
            mockdata.user_id = value.int(ln, col)? as u64;
            flags &= !0b0100000000;
          }
          "is_active" => {
            mockdata.is_active = value.bool(ln, col)?;
            flags &= !0b0010000000;
          }
          "mock_data" => {
            mockdata.mock_data = Vec::parse_json(value)?;
            flags &= !0b0001000000;
          }
          "user_details" => {
            mockdata.user_details = UserDetails::parse_json(value)?;
            flags &= !0b0000100000;
          }
          "related_users" => {
            mockdata.related_users = Vec::parse_json(value)?;
            flags &= !0b0000010000;
          }
          "last_login" => {
            mockdata.last_login = value.string(ln, col)?;
            flags &= !0b0000001000;
          }
          "website" => {
            mockdata.website = value.string(ln, col)?;
            flags &= !0b0000000100;
          }
          "validation_regex" => {
            mockdata.validation_regex = value.string(ln, col)?;
            flags &= !0b0000000010;
          }
          "binary_data" => {
            mockdata.binary_data = value.string(ln, col)?;
            flags &= !0b0000000001;
          }
          _ => (),
        }

        if flags == 0 {
          break;
        }
      }

      return Ok(mockdata);
    }
    Err(json::JsonError::NoMatch)
  }
}
