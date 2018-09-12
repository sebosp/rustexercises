//! json.rs has a series of utils for dealing with JSON out of simple XMLs.

use std::fmt;
/// `JsonDataType` structure contains the representation of KV pairs and internal arrays.
/// Examples of data:
/// {"key":"string"}
/// {"key":["0","1"]}
/// {"key":[{"internal_key":"internal_value"}]
/// The only supported atomic type is String
pub enum JsonDataType {
 Empty,
 Array(Vec<Box<JsonData>>),
 Object(Vec<(String,Box<JsonData>)>),
 Atomic(String),
}
pub struct JsonData {
  pub data: Box<JsonDataType>,
  //pub parent: Box<JsonDataType>,
}

/// Implement the Display Trait for printing.
impl fmt::Display for JsonDataType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"{}",self.to_string())
  }
}

impl JsonData {
  pub fn new() -> Self {
    JsonData { data: Box::new( JsonDataType::Empty )}
  }
  /// `to_string` returns the string representation of a JsonDataType object.
  pub fn to_string(&self) -> String {
    match *self.data {
      JsonDataType::Atomic(data) => format!("\"{}\"", data),
      JsonDataType::Array(array_data) => {
        format!("[{}]",
                array_data
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(","))
      }
      JsonDataType::Object(obj_data) => {
        format!("{{{}}}",
                obj_data
                .iter()
                .map(|(k,v)| format!("\"{}\":{}",k,v.to_string()))
                .collect::<Vec<_>>().join(","))
      },
      JsonDataType::Empty => "{}".to_owned(),
    }
  }
  /// `from_xml_string` returns a JSON version of an XML string.
  /// If the XML is not valid returns Err.
  pub fn from_xml_string(data: &String, worker_id_string: &String, verbosity: i8) -> Result<Self,String> {
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
    Ok(JsonData { data: Box::new(JsonDataType::Atomic("Unimplemented".to_owned())) })
  }
  /// `init` a string at the current level, if the current level has a String
  /// already it transforms the enum to JsonDataType::Array
  pub fn init(&mut self, input: String) {
    match *self.data {
      JsonDataType::Empty => {
        self.data = Box::new(JsonDataType::Atomic(input));
      },
      JsonDataType::Atomic(data) => {
        self.data = Box::new(
          JsonDataType::Array(vec![
            Box::new(JsonData{ data: JsonDataType::Atomic(data)}),
            Box::new(JsonData{ data: JsonDataType::Atomic(input)})
          ])
        );
      },
      _ => {},
    }
  }
  /// `push_value` to the current hierarchy.
  pub fn push_value(&mut self, input: String) {
    match self {
      _ => {},
      JsonDataType::Array(ref mut array_data) => {
        array_data.push(Box::new(JsonDataType::Atomic(input)));
      },
      JsonDataType::Object(ref mut obj_data) => {
        obj_data.push((input, Box::new(JsonDataType::Empty)));
      }
    }
  }
  /// `insert` to the current hierarchy.
  pub fn insert(&mut self, input: String) {
    let should_init = match self {
      JsonDataType::Empty | JsonDataType::Atomic(_) => { true },
      _ => false,
    };
    if should_init {
      self.init(input);
    } else {
      self = self.insert(input);
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  #[ignore]
  fn it_transforms_xml_to_json() {
    assert_eq!(
      JsonDataType::from_xml_string(&"<div><li>1</li></div>".to_owned(),&"TEST".to_owned(),0i8).unwrap().to_string(),
      "{\"div\":{\"li\":\"1\"}}".to_owned()
    );
    assert_eq!(
      JsonDataType::from_xml_string(&"<li>1</li>".to_owned(),&"TEST".to_owned(),0i8).unwrap().to_string(),
      "{\"li\":\"1\"}".to_owned()
    );
    assert_eq!(
      JsonDataType::from_xml_string(&"<div><li>1</li><li>2</li></div>".to_owned(),&"TEST".to_owned(),0i8).unwrap().to_string(),
      "{\"div\":{\"li\":[\"1\",\"2\"]}}".to_owned()
    );
    assert_eq!(
      JsonDataType::from_xml_string(&"<body><div><li>1</li></div><div><li>2</li></div></body>".to_owned(),&"TEST".to_owned(),0i8).unwrap().to_string(),
      "{\"items\":{\"div\":[{\"li\":\"1\"},{\"li\":\"2\"}]}}".to_owned()
    );
    assert_eq!(
      JsonDataType::from_xml_string(&"<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned(),&"TEST".to_owned(),0i8).unwrap().to_string(),
      "{\"div\":{\"li\":[\"1\",\"2\"],\"lt\":\"a\"}}".to_owned()
    );
  }
  #[test]
  fn it_transforms_to_string() {
    // Technically this could be invalid JSON but not for our simple case.
    let simple_string = JsonDataType::Atomic("String".to_owned());
    assert_eq!(simple_string.to_string(),"\"String\"".to_owned());
    let simple_array1 =
      JsonDataType::Array(
        vec![
          Box::new(JsonDataType::Atomic("String".to_owned())),
        ]
      );
    assert_eq!(simple_array1.to_string(),"[\"String\"]".to_owned());
    let simple_array2 =
      JsonDataType::Array(
        vec![
          Box::new(JsonDataType::Atomic("String1".to_owned())),
          Box::new(JsonDataType::Atomic("String2".to_owned())),
        ]
      );
    assert_eq!(simple_array2.to_string(),"[\"String1\",\"String2\"]".to_owned());
    let simple_obj =
      JsonDataType::Object(
        vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value".to_owned())))]
      );
    assert_eq!(simple_obj.to_string(),"{\"Key\":\"Value\"}".to_owned());
    let multi_kv =
      JsonDataType::Object(
        vec![
          ("Key1".to_owned(), Box::new(JsonDataType::Atomic("Value1".to_owned()))),
          ("Key2".to_owned(), Box::new(JsonDataType::Atomic("Value2".to_owned())))
        ]
      );
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":\"Value2\"}".to_owned());
    let array_contains_objs =
      JsonDataType::Array(
        vec![
          Box::new(JsonDataType::Object(
            vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value1".to_owned())))]
          )),
          Box::new(JsonDataType::Object(
            vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value2".to_owned())))])
          ),
        ]
      );
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"}]".to_owned());
    let obj_contains_array =
      JsonDataType::Object(
        vec![("Key".to_owned(), Box::new(JsonDataType::Array(
          vec![
            Box::new(JsonDataType::Atomic("Value1".to_owned())),
            Box::new(JsonDataType::Atomic("Value2".to_owned()))
          ]))
        )]
      );
    assert_eq!(obj_contains_array.to_string(),"{\"Key\":[\"Value1\",\"Value2\"]}".to_owned());
  }
  #[test]
  fn it_inserts() {
    // Technically this could be invalid JSON but not for our simple case.
    let mut simple_string = JsonDataType::new();
    simple_string.insert("String".to_owned());
    assert_eq!(simple_string.to_string(),"\"String\"".to_owned());
    let mut simple_array2 = simple_string;
    simple_array2.insert("String".to_owned());
    assert_eq!(simple_array2.to_string(),"[\"String1\",\"String2\"]".to_owned());
    let simple_obj =
      JsonDataType::Object(
        vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value".to_owned())))]
      );
    assert_eq!(simple_obj.to_string(),"{\"Key\":\"Value\"}".to_owned());
    let multi_kv =
      JsonDataType::Object(
        vec![
          ("Key1".to_owned(), Box::new(JsonDataType::Atomic("Value1".to_owned()))),
          ("Key2".to_owned(), Box::new(JsonDataType::Atomic("Value2".to_owned())))
        ]
      );
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":\"Value2\"}".to_owned());
    let array_contains_objs =
      JsonDataType::Array(
        vec![
          Box::new(JsonDataType::Object(
            vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value1".to_owned())))]
          )),
          Box::new(JsonDataType::Object(
            vec![("Key".to_owned(), Box::new(JsonDataType::Atomic("Value2".to_owned())))])
          ),
        ]
      );
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"}]".to_owned());
    let obj_contains_array =
      JsonDataType::Object(
        vec![("Key".to_owned(), Box::new(JsonDataType::Array(
          vec![
            Box::new(JsonDataType::Atomic("Value1".to_owned())),
            Box::new(JsonDataType::Atomic("Value2".to_owned()))
          ]))
        )]
      );
    assert_eq!(obj_contains_array.to_string(),"{\"Key\":[\"Value1\",\"Value2\"]}".to_owned());
  }
}
