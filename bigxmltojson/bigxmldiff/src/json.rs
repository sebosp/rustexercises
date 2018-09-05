//! json.rs has a series of utils for dealing with JSON out of simple XMLs.
/// `xml_to_json` returns a JSON version of an XML chunk.
/// If the XML is not valid returns Err.
pub fn xml_to_json(data: &String, worker_id_string: &String, verbosity: i8) -> Result<String,String> {
  let mut inside_tag = false;
  let mut cur_tag = String::with_capacity(128);
  let mut cur_tag_content = String::with_capacity(128);
  for cur_char in data.chars() {
    if cur_char == '<' {
      inside_tag = true;
      continue;
    }
    if cur_char == '>' {
      if verbosity > 3 {
        println!("WORKER[{}] Found tag '{}'", worker_id_string, cur_tag);
      }
      cur_tag_content.truncate(0);
      cur_tag.truncate(0);
      inside_tag = false;
      continue;
    }
    if inside_tag {
      cur_tag.push(cur_char);
      if verbosity > 3 {
        println!("WORKER[{}] Adding to cur_tag: '{}': {}", worker_id_string, cur_char, cur_tag);
      }
    } else {
      cur_tag_content.push(cur_char);
      if verbosity > 3 {
        println!("WORKER[{}] Adding to cur_tag_content: '{}': {}", worker_id_string, cur_char, cur_tag_content);
      }
    }
  }
  Ok("".to_owned())
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_transforms_xml_to_json() {
    assert_eq!(
      xml_to_json("<div><li>1</li></div>".to_owned(),"TEST".to_owned(),0i8),
      Ok("{\"div\":{\"li\":\"1\"}}".to_owned())
    );
    assert_eq!(
      xml_to_json("<li>1</li>".to_owned(),"TEST".to_owned(),0i8),
      Ok("{\"li\":\"1\"}".to_owned())
    );
    assert_eq!(
      xml_to_json("<div><li>1</li><li>2</li></div>".to_owned(),"TEST".to_owned(),0i8),
      Ok("{\"div\":{\"li\":[\"1\",\"2\"]}}".to_owned())
    );
    assert_eq!(
      xml_to_json("<body><div><li>1</li></div><div><li>2</li></div></body>".to_owned(),"TEST".to_owned(),0i8),
      Ok("{\"items\":{\"div\":[{\"li\":\"1\"},{\"li\":\"2\"}]}}".to_owned())
    );
    assert_eq!(
      xml_to_json("<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned(),"TEST".to_owned(),0i8),
      Ok("{\"div\":{\"li\":[\"1\",\"2\"],\"lt\":\"a\"}}".to_owned())
    );
  }
}
