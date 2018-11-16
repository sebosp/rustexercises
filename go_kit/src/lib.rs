//! StringService provides operations on strings.
//! This file contains the business logic of our microservices
// import "context"

extern crate jsonrpc_core;
// jsonrpc_macros are broken on nightly (https://github.com/paritytech/jsonrpc/issues/328)
// #[macro_use]
// extern crate jsonrpc_macros;

// The RequestResponse is handled by using serde_json
pub mod jsonrpc;
use jsonrpc_core::*;

pub struct StringService{
    rpc: IoHandler
}

impl StringService {
    fn uppercase(input: String) -> Result<String, String> {
    if input== "" {
            Err("Empty String".to_owned())
    } else {
        Ok(input.to_ascii_uppercase())
    }
    }
    fn count(input: String) -> usize {
        input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_uppercases_filled_string() {
        let test_uppercase = StringService::uppercase("test".to_owned());
        assert_eq!(test_uppercase,Ok("TEST".to_owned()));
    }

    #[test]
    fn it_uppercases_empty_string() {
        let test_uppercase = StringService::uppercase("".to_owned());
        assert_eq!(test_uppercase,Err("Empty String".to_owned()));
    }

    #[test]
    fn it_counts_filled_string() {
        let test_count = StringService::count("test".to_owned());
        assert_eq!(test_count,4usize);
    }

    #[test]
    fn it_counts_empty_string() {
        let test_count = StringService::count("".to_owned());
        assert_eq!(test_count,0usize);
    }
}