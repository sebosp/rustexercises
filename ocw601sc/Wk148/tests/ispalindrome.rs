extern crate ispalindrome;
use ispalindrome::*;
#[cfg(test)]
#[test]
fn test_ispalindrome() {
  assert_eq!(ispalindrome(&"race car".to_string()),false);
  assert_eq!(ispalindrome(&"racecar".to_string()),true);
  assert_eq!(ispalindrome(&"áoá".to_string()),true);
  assert_eq!(ispalindrome(&"áooá".to_string()),true);
}
