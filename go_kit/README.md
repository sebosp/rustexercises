# GoKit Traits

Playing around with http://gokit.io/ layers as Traits.

## Initial implementation
http://gokit.io/examples/stringsvc.html

### Business Logic:
The business logic lives in `src/lib.rs`

### JSON Requests and responses
The Requests Response of the Struct data is handled by using derives:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct StringService
```
The _ser_ialization and _de_serialization comes from the crate `serde_derive`
The Request and Response from the Struct impl is handled by using Parity JSON-RPC
