extern crate v2;
use v2::*;
fn main() {
    println!("Enter X:");
    let x = read_f64();
    println!("Enter Y:");
    let y = read_f64();
    let input = V2::new(x,y);
    println!("{}",input);
}
