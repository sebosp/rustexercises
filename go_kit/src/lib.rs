//! StringService provides operations on strings.
//! This file contains the business logic of our microservices
// import "context"

// The RequestResponse is handled by using rocket_contrib::Json
pub mod json;

// use rocket_contrib::json::Json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct StringService {
  data: String
}

impl StringService {
  pub fn new(data: String) -> StringService {
    StringService{
      data: data
    }
  }
	pub fn uppercase(&self) -> Result<String, String> {
    if self.data == "" {
		 Err("Empty String".to_owned())
    } else {
	    Ok(self.data.to_ascii_uppercase())
    }
  }
	pub fn count(&self) -> usize {
    self.data.len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_uppercases_filled_string() {
      let test_service = StringService::new("test".to_owned());
      assert_eq!(test_service.uppercase(),Ok("TEST".to_owned()));
  }

  #[test]
  fn it_uppercases_empty_string() {
      let test_service = StringService::new("".to_owned());
      assert_eq!(test_service.uppercase(),Err("Empty String".to_owned()));
  }

    #[test]
  fn it_counts_filled_string() {
      let test_service = StringService::new("test".to_owned());
      assert_eq!(test_service.count(),4usize);
  }

  #[test]
  fn it_counts_empty_string() {
      let test_service = StringService::new("".to_owned());
      assert_eq!(test_service.count(),0usize);
  }
}