extern crate additionals;
use additionals::*;
#[cfg(test)]
#[test]
fn test_is_palindrome() {
  assert_eq!(is_palindrome(&"race car".to_string()),false);
  assert_eq!(is_palindrome(&"racecar".to_string()),true);
  assert_eq!(is_palindrome(&"áoá".to_string()),true);
  assert_eq!(is_palindrome(&"áooá".to_string()),true);
}
