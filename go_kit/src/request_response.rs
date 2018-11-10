use rocket_contrib::json::Json;

#[derive(Deserialize)]
struct RequestResponse {
  input: String,
  output: String,
}
pub trait RequestResponse {
    // Every method will be modeled as a remote procedure call.
   // For each method, we define request and response functions,
   // capturing all of the input and output parameters respectively.
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