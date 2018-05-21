extern crate additionals;
use additionals::*;
#[cfg(test)]
#[test]
fn test_extract_tags() {
  assert_eq!(extract_tags(&"[aoeu]".to_string()),Ok(vec!["aoeu".to_string()]));
  assert_eq!(extract_tags(&"aoeu".to_string()),Ok(vec![]));
  assert_eq!(extract_tags(&"aoeu]".to_string()),Err("Unmatched open tag".to_string()));
  assert_eq!(extract_tags(&"[aoeu".to_string()),Err("Unmatched close tag".to_string()));
  assert_eq!(extract_tags(&"[[aoeu]]".to_string()),Err("Tags within tags are not supported".to_string()));
  assert_eq!(extract_tags(
    &"[aoeu1][aoeu2] foo [aoeu3] bar".to_string()),
    Ok(vec![
       "aoeu1".to_string(),
       "aoeu2".to_string(),
       "aoeu3".to_string()
    ])
  );
}
