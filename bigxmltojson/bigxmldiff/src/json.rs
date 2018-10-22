extern crate treexml;
extern crate serde_json;
extern crate node2object;

pub fn xml_to_json(input: &String) -> Result<String,String> {
  let tree = treexml::Document::parse(input.as_bytes());
  match tree {
    Ok(v) => {
      let dom_root = v.root.unwrap();
      let json_rep = serde_json::Value::Object(node2object::node2object(&dom_root));
      Ok(json_rep.to_string())
    },
    Err(e) => {
      Err(e.description().to_string())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_transforms_from_xml_string() {
    assert_eq!(
      xml_to_json(&"<li>1</li>".to_owned()).unwrap(),
      "{\"li\":1.0}".to_owned()
    );
    assert_eq!(
      xml_to_json(&"<div><li>1</li></div>".to_owned()).unwrap(),
      "{\"div\":{\"li\":1.0}}".to_owned()
    );
    assert_eq!(
      xml_to_json(&"<div><li>1</li><li>2</li></div>".to_owned()).unwrap(),
      "{\"div\":{\"li\":[1.0,2.0]}}".to_owned()
    );
    assert_eq!(
      xml_to_json(&"<body><div><li>1</li></div><div><li>2</li></div></body>".to_owned()).unwrap(),
      "{\"body\":{\"div\":[{\"li\":1.0},{\"li\":2.0}]}}".to_owned()
    );
    assert_eq!(
      xml_to_json(&"<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned()).unwrap(),
      "{\"div\":{\"li\":[1.0,2.0],\"lt\":\"a\"}}".to_owned()
    );
  }
}
