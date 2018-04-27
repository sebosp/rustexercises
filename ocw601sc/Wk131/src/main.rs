extern crate fibo;
use fibo::*;
fn main() {
    println!("Enter n: ");
    println!("Result: {}",Fibonacci::new(&"seq".to_string(),read_u8()).solve());
}
