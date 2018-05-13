extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;
use std::io;
pub fn is_palindrome(input: &str) -> bool {
  let graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  graphemes.iter().take(graphemes.len() + 1 / 2 as usize).zip(graphemes.iter().rev().take(graphemes.len() + 1 / 2 as usize)).all(|(x,y)| x == y)
}
pub fn is_substring(input: &str, fragment: &str) -> bool {
  let input_graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  let fragment_graphemes = UnicodeSegmentation::graphemes(fragment, true).collect::<Vec<&str>>();
  let mut match_offset = 0;
  for (input_index,input_grapheme) in input_graphemes.iter().enumerate() {
    if fragment.len() - match_offset > input_graphemes.len() - input_index {
      return false
    }
    if input_grapheme == &fragment_graphemes[match_offset] {
      match_offset += 1;
    }
    if match_offset == fragment.len() {
      return true
    }
  }
  false
}
/// Helper functions
pub fn read_line() -> String {
  let mut input = String::new();
  io::stdin().read_line(&mut input)
    .expect("Failed to read line");
  input
}
