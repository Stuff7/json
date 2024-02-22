use json::{Json, JsonError, JsonParse};
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

#[derive(Debug, JsonParse)]
enum Salutation {
  Hello,
  Hey,
  Whatsup,
}

#[derive(Debug, JsonParse)]
struct Preferences {
  theme: Box<str>,
  language: Box<str>,
}

#[derive(Debug, JsonParse)]
struct UserDetails {
  email: Box<str>,
  phone_number: Box<str>,
  is_verified: bool,
  preferences: Preferences,
}

#[derive(Debug, JsonParse)]
struct RelatedUser {
  username: Box<str>,
  user_id: u64,
  is_active: bool,
  salut: Salutation,
}

#[derive(Debug, JsonParse)]
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
