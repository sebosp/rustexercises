extern crate additionals;
use additionals::*;
fn main() {
    println!("Enter a string to check for palindrome:");
    println!("ispalindrome={}",is_palindrome(&read_line()));
}
