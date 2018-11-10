//! StringService provides operations on strings.
//! This file contains the business logic of our microservices
// import "context"

pub mod request_response;
pub struct StringService {}

impl StringService{
	pub fn uppercase(&self, input: String) -> Result<String, String> {
    if input == "" {
		 Err("Empty String".to_owned())
    } else {
	    Ok(input.to_ascii_uppercase())
    }
  }
	pub fn count(&self, input: String) -> i64 {
    input.len()
  }
}
