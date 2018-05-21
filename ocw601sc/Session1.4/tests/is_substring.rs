extern crate additionals;
use additionals::*;
#[cfg(test)]
#[test]
fn test_is_substring() {
  assert_eq!(is_substring(&"barfoobar".to_string(),&"foo".to_string()),true);
  assert_eq!(is_substring(&"foo".to_string(),&"barfoobar".to_string()),false);
  assert_eq!(is_substring(&"something".to_string(),&"thing".to_string()),true);
  assert_eq!(is_substring(&"something".to_string(),&"things".to_string()),false);
  assert_eq!(is_substring(&"something".to_string(),&"some".to_string()),true);
}
