extern crate polynomial;
use polynomial::*;
fn main() {
    let testfrom = Polynomial::from_string("1 2 3".to_string());
    println!("{}",testfrom.to_string());
}
