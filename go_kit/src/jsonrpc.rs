use super::*;
use jsonrpc_core::*;
// build_rpc_trait! {
// 	pub trait Rpc {
// 		// Uppercases a string
// 		#[rpc(name = "uppercase")]
// 		fn uppercase(input: String) -> Result<String,String>;

// 		// Counts the number of characters
// 		#[rpc(name = "count")]
// 		fn count(input: String) -> Result<usize, String>;
// 	}
// }

pub fn setup_rpc() -> IoHandler {
	let mut io = IoHandler::default();
	
	io.add_method("uppercase", |params: Params| {
		let uppercase = match params.parse().unwrap() {
			jsonrpc_core::Params::Array(a) => StringService::uppercase(a[0].to_string()),
			jsonrpc_core::Params::Map(m) => StringService::uppercase(m["key"].to_string()),
			_ => StringService::uppercase("unknown".to_string()),
		};
		match uppercase {
			Ok(value) => Ok(Value::String(value)),
			Err(err) => Ok(Value::String(err)),
		}
	});
	io
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_uppercases_filled_string() {
		let io = setup_rpc();
	  let request = r#"{"jsonrpc": "2.0", "method": "uppercase", "params": ["test"], "id": 1}"#;
	  let response = r#"{"jsonrpc":"2.0","result":"\"TEST\"","id":1}"#;
    assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
  }
}

// type uppercaseRequest struct {
// 	S string `json:"s"`
// }

// type uppercaseResponse struct {
// 	V   string `json:"v"`
// 	Err string `json:"err,omitempty"` // errors don't JSON-marshal, so we use a string
// }

// type countRequest struct {
// 	S string `json:"s"`
// }

// type countResponse struct {
// 	V int `json:"v"`
// }