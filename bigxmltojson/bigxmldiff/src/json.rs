//! json.rs has a series of utils for dealing with JSON out of simple XMLs.

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Weak;
/// `JsonDataType` structure contains the representation of KV pairs and internal arrays.
/// Examples of data:
/// {"key":"string"}
/// {"key":["0","1"]}
/// {"key":[{"internal_key":"internal_value"}]
/// The only supported atomic type is String
pub enum JsonDataType {
 Empty,
 Array(Vec<JsonData>),
 Object(Vec<(String,JsonData)>),
 Atomic(String),
}
pub struct JsonData {
  pub data: Rc<RefCell<JsonDataType>>,
  pub parent: Option<Weak<RefCell<JsonDataType>>>,
}

/// Implement the Display Trait for printing.
impl fmt::Display for JsonData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"{}",self.to_string())
  }
}

impl JsonData {
  pub fn new() -> Self {
    JsonData { data: Rc::new(RefCell::new(JsonDataType::Empty))}
  }
  /// `to_string` returns the string representation of a JsonDataType object.
  pub fn to_string(&self) -> String {
    match *self.data {
      JsonDataType::Atomic(ref data) => format!("\"{}\"", data),
      JsonDataType::Array(ref array_data) => {
        format!("[{}]",
                array_data
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(","))
      }
      JsonDataType::Object(ref obj_data) => {
        format!("{{{}}}",
                obj_data
                .iter()
                .map(|(k,v)| format!("\"{}\":{}",k,v.to_string()))
                .collect::<Vec<_>>().join(","))
      },
      JsonDataType::Empty => "null".to_owned(),
    }
  }
  /// `from_xml_string` returns a JSON version of an XML string.
  /// If the XML is not valid returns Err.
  pub fn from_xml_string(data: &String, worker_id_string: &String, verbosity: i8) -> Result<Self,String> {
    let mut inside_tag = false;
    let mut cur_tag = String::with_capacity(128);
    let mut cur_tag_content = String::with_capacity(128);
    let mut res = JsonData::new();
    for cur_char in data.chars() {
      if cur_char == '<' {
        inside_tag = true;
        continue;
      }
      if cur_char == '>' {
        if verbosity > 3 {
          println!("WORKER[{}] Found tag '{}'", worker_id_string, cur_tag);
        }
        if cur_tag.starts_with('/') {
          cur_tag.drain(..1);
          println!("WORKER[{}] Current tag '{{{}:{}}} '", worker_id_string, cur_tag, cur_tag_content);
          res.insert_kv(cur_tag.clone(),cur_tag_content.clone());
          if cur_tag_content.len() > 0 {
            // XXX: Continue
          } else {
          }
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
    Ok(JsonData { data: Rc::new(JsonDataType::Atomic("Unimplemented".to_owned())) })
  }
  /// `transform_to_atomic` an entry currently Empty variant to an Atomic variant.
  pub fn transform_to_atomic(&mut self, input: String) {
    let mut temp = JsonData::new();
    match *self.data {
      JsonDataType::Empty => {
        temp.data = Rc::new(JsonDataType::Atomic(input));
      },
      _ => {
        panic!("transform_to_atomic supports only Empty JsonDataType");
      },
    }
    self.data = temp.data;
  }
  /// `push_value` to the current hierarchy.
  pub fn push_value(&mut self, input: String) {
    match *self.data {
      JsonDataType::Array(ref mut array_data) => {
        array_data.push(JsonData{ data: Rc::new(JsonDataType::Atomic(input))});
      },
      JsonDataType::Object(ref mut obj_data) => {
        let last_index = obj_data.len();
        //obj_data.push((input, JsonData{ data: Rc::new(JsonDataType::Empty)}));
        obj_data[last_index-1].1.insert(input);
      },
      _ => {},
    }
  }
  /// `insert` to the current level
  pub fn insert(&mut self, input: String) {
    match *self.data {
      JsonDataType::Empty => {
        self.transform_to_atomic(input);
        return;
      },
      JsonDataType::Atomic(_) => {
        // XXX: How does one decide if something is to be transformed to Array or to Object?
        self.data =
          Rc::new(
            JsonDataType::Array(vec![
              JsonData{ data: Rc::clone(&self.data)},
            ])
          );
        self.push_value(input);
      },
      JsonDataType::Array(_) => {
        self.push_value(input);
        return;
      },
      JsonDataType::Object(_) => {
        panic!("Unsupported type for insert, use insert_kv");
      },
    };
  }
  /// `insert_kv` Inserts a Key/Value pair.
  pub fn insert_kv(&mut self, input_key: String, input_value: String) {
    // If the key exists, add the input_value to the key.
    let data = &*self.data;
    match data {
      JsonDataType::Object(ref mut kv_array) => {
        for (k,v) in kv_array.iter_mut() {
          if *k == input_key {
            v.insert(input_value.to_owned());
            return;
          }
        }
        kv_array.push((input_key.to_owned(), JsonData {
          data: Rc::new(JsonDataType::Atomic(input_value.to_owned()))
        }));
      },
      _ => {
        panic!("Unsupported type for insert_kv, use insert");
      },
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_transforms_from_xml_string() {
    let verbosity = 4i8;
    assert_eq!(
      JsonData::from_xml_string(&"<div><li>1</li></div>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"div\":{\"li\":\"1\"}}".to_owned()
    );
    assert_eq!(
      JsonData::from_xml_string(&"<li>1</li>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"li\":\"1\"}".to_owned()
    );
    assert_eq!(
      JsonData::from_xml_string(&"<div><li>1</li><li>2</li></div>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"div\":{\"li\":[\"1\",\"2\"]}}".to_owned()
    );
    assert_eq!(
      JsonData::from_xml_string(&"<body><div><li>1</li></div><div><li>2</li></div></body>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"items\":{\"div\":[{\"li\":\"1\"},{\"li\":\"2\"}]}}".to_owned()
    );
    assert_eq!(
      JsonData::from_xml_string(&"<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"div\":{\"li\":[\"1\",\"2\"],\"lt\":\"a\"}}".to_owned()
    );
  }
  #[test]
  fn it_transforms_to_string() {
    // Technically this could be invalid JSON but not for our simple case.
    let simple_string = JsonData{ data: Rc::new(JsonDataType::Atomic("String".to_owned()))};
    assert_eq!(simple_string.to_string(),"\"String\"".to_owned());
    let simple_array1 = JsonData{ data: Rc::new(
      JsonDataType::Array(
        vec![
          JsonData{ data: Rc::new(JsonDataType::Atomic("String".to_owned()))},
        ]
      ))};
    assert_eq!(simple_array1.to_string(),"[\"String\"]".to_owned());
    let simple_array2 = JsonData{ data: Rc::new(
      JsonDataType::Array(
        vec![
          JsonData{ data: Rc::new(JsonDataType::Atomic("String1".to_owned()))},
          JsonData{ data: Rc::new(JsonDataType::Atomic("String2".to_owned()))},
        ]
      ))};
    assert_eq!(simple_array2.to_string(),"[\"String1\",\"String2\"]".to_owned());
    let simple_obj = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value".to_owned()))})]
      ))};
    assert_eq!(simple_obj.to_string(),"{\"Key\":\"Value\"}".to_owned());
    let multi_kv = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![
          ("Key1".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))}),
          ("Key2".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value2".to_owned()))})
        ]
      ))};
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":\"Value2\"}".to_owned());
    let array_contains_objs = JsonData{ data: Rc::new(
      JsonDataType::Array(
        vec![
          JsonData{ data: Rc::new(JsonDataType::Object(
            vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))})]
          ))},
          JsonData{ data: Rc::new(JsonDataType::Object(
            vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value2".to_owned()))})])
          )},
        ]
      ))};
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"}]".to_owned());
    let obj_contains_array = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Array(
          vec![
            JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))},
            JsonData{ data: Rc::new(JsonDataType::Atomic("Value2".to_owned()))}
          ]))}
        )]
      ))};
    assert_eq!(obj_contains_array.to_string(),"{\"Key\":[\"Value1\",\"Value2\"]}".to_owned());
  }
  #[test]
  fn it_inserts() {
    // Technically this could be invalid JSON but not for our simple case.
    let mut simple_string = JsonData::new();
    simple_string.insert("String1".to_owned());
    assert_eq!(simple_string.to_string(),"\"String1\"".to_owned());
    let mut simple_array2 = simple_string;
    simple_array2.insert("String2".to_owned());
    assert_eq!(simple_array2.to_string(),"[\"String1\",\"String2\"]".to_owned());
    let mut simple_obj = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value".to_owned()))})]
      ))};
    simple_obj.insert("Value2".to_owned());
    assert_eq!(simple_obj.to_string(),"{\"Key\":[\"Value\",\"Value2\"]}".to_owned());
    let mut multi_kv = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![
          ("Key1".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))}),
          ("Key2".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value2.0".to_owned()))})
        ]
      ))};
    multi_kv.insert("Value2.1".to_owned());
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":[\"Value2.0\",\"Value2.1\"]}".to_owned());
    let mut array_contains_objs = JsonData{ data: Rc::new(
      JsonDataType::Array(
        vec![
          JsonData{ data: Rc::new(JsonDataType::Object(
            vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))})]
          ))},
          JsonData{ data: Rc::new(JsonDataType::Object(
            vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Atomic("Value2".to_owned()))})])
          )},
        ]
      ))};
    array_contains_objs.insert("Value".to_owned());
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"},\"Value\"]".to_owned());
    let mut obj_contains_array = JsonData{ data: Rc::new(
      JsonDataType::Object(
        vec![("Key".to_owned(), JsonData{ data: Rc::new(JsonDataType::Array(
          vec![
            JsonData{ data: Rc::new(JsonDataType::Atomic("Value1".to_owned()))},
            JsonData{ data: Rc::new(JsonDataType::Atomic("Value2".to_owned()))}
          ]))}
        )]
      ))};
    obj_contains_array.insert("Value3".to_owned());
    assert_eq!(obj_contains_array.to_string(),"{\"Key\":[\"Value1\",\"Value2\",\"Value3\"]}".to_owned());
  }
}
