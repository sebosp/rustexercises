#[macro_use]
extern crate clap; 

pub mod command;
fn main() {
    command::parse_arguments();
    println!("Hello, world!");
}