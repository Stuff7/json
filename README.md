# Json

JSON parser in rust with no dependencies

# Usage

```rust
use json::{Json, JsonParse};

#[derive(Debug, JsonParse)] // Derive JsonParse
struct YourStruct {
  date_created: Box<str>,
  #[json(alias = "date_modified")] // Use this attribute if you need the field to have a different name from the json key
  modified: Box<str>,
  inner: InnerStruct, // Inner structs must derive JsonParse too
  id: usize,
  name: Box<str>,
  flag: bool,
  optional: Option<f64>,
}

#[derive(Debug, JsonParse)]
struct InnerStruct {
  date_modified: Box<str>,
  version: Box<str>,
  id: isize,
  color: Color, // Inner enums must derive JsonParse too
}

#[derive(Debug, JsonParse)]
enum Color {
  Red, // Parses from "Red"
  #[json(alias = "g")] // Parses from "g"
  Green,
  Blue,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let filename: String = std::env::args().nth(1).ok_or("Usage json <file>")?;
  let bytes: Vec<u8> = std::fs::read(filename)?;
  let your_struct: YourStruct = bytes.json()?; // json method comes from Json trait
  dbg!(&your_struct);
  Ok(())
}
```
