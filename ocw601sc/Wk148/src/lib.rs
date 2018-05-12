extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;
use std::io;
pub fn ispalindrome(input: &str) -> bool {
  let graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  graphemes.iter().take(graphemes.len() + 1 / 2 as usize).zip(graphemes.iter().rev().take(graphemes.len() + 1 / 2 as usize)).all(|(x,y)| x == y)
}
/// Helper functions
pub fn read_line() -> String {
  let mut input = String::new();
  io::stdin().read_line(&mut input)
    .expect("Failed to read line");
  input
}
