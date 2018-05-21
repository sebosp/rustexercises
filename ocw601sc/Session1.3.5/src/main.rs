extern crate polynomial;
use polynomial::*;
fn main() {
    let poly = Polynomial::from_string(read_line());
    println!("{}",poly.to_string());
    //println!("{}",poly.solve(f64::from(read_line())));
}
