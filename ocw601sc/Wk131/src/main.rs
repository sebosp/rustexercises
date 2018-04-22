extern crate fibo;
use fibo::*;
fn main() {
    println!("Enter n: ");
    let calculation = fibo::Fibonacci {
        target: fibo::FibonacciSolutions::Sequential(read_u8())
    };
    println!("Result: {}",calculation.solve());
}
