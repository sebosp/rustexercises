//! json.rs has a series of utils for dealing with JSON out of simple XMLs.

use std::fmt;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
/// `JsonDataType` structure contains the representation of KV pairs and internal arrays.
/// Examples of data:
/// {"key":"string"}
/// {"key":["0","1"]}
/// {"key":[{"internal_key":"internal_value"}]
/// The only supported atomic type is String
pub enum JsonDataType {
  Empty,
  Atomic(RefCell<String>),
  Array(RefCell<Vec<JsonData>>),
  Object(RefCell<Vec<(String,JsonData)>>),
}
pub struct JsonData {
  pub data: Rc<JsonDataType>,
  pub parent: RefCell<Weak<JsonData>>,
}

/// Implement the Display Trait for printing.
impl fmt::Display for JsonData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"{}",self.to_string())
  }
}

impl JsonDataType {
  /// `new` Creates an Empty JsonDataType by default
  pub fn new() -> JsonDataType {
    JsonDataType::Empty
  }
  /// `new_atomic_from_string` Creates a JsonDataType String input
  pub fn new_atomic_from_string(input: String) -> JsonDataType {
    JsonDataType::Atomic(
      RefCell::new(input)
    )
  }
  /// `new_array_from_string` Creates a JsonDataType Array from a String input
  pub fn new_array_from_string(input: String) -> JsonDataType {
    JsonDataType::Array(
      RefCell::new(
        vec![
          JsonData::new_atomic_from_string(input)
        ]
      )
    )
  }
  /// `new_object_from_kv_strinsg` Creates a JsonDataType Object from
  /// String Key,Value pair.
  pub fn new_object_from_kv_strings(input_key: String, input_value: String) -> JsonDataType {
    JsonDataType::Object(
      RefCell::new(
        vec![
          (input_key,JsonData::new_atomic_from_string(input_value))
        ]
      )
    )
  }
}
impl JsonData {
  /// `new` returns a new JsonData structure.
  /// XXX: Should this be Rc<> ?
  /// # Example
  /// ```
  /// let empty_json = JsonData::new("String".to_owned());
  /// assert_eq!(empty_json.to_string(),"\"\"".to_owned());
  /// ```
  pub fn new() -> Self {
    JsonData {
      data: Rc::new(
              JsonDataType::new()
            ),
      parent: RefCell::new(Weak::new()),
    }
  }
  /// `new_atomic_from_string` creates a JsonData of JsonDataType Atomic for
  /// an input String.
  /// # Example
  /// ```
  /// let simple_string = JsonData::new_atomic_from_string("String".to_owned());
  /// assert_eq!(simple_string.to_string(),"\"String\"".to_owned());
  /// ```
  pub fn new_atomic_from_string(input: String) -> JsonData {
    JsonData {
      data: Rc::new(
              JsonDataType::new_atomic_from_string(input)
            ),
      parent: RefCell::new(Weak::new()),
    }
  }
  /// `new_array_from_string` creates a JsonData of JsonDataType Array for
  /// an input String.
  /// # Example
  /// ```
  /// let simple_array = JsonData::new_array_from_string("String".to_owned());
  /// assert_eq!(simple_array.to_string(),"[\"String\"]".to_owned());
  /// ```
  pub fn new_array_from_string(input: String) -> JsonData {
    JsonData {
      data: Rc::new(
              JsonDataType::new_array_from_string(input)
            ),
      parent: RefCell::new(Weak::new()),
    }
  }
  /// `new_object_from_kv_strings` creates a JsonData of JsonDataType Array for
  /// an input String.
  /// # Example
  /// ```
  /// let simple_obj = JsonData::new_object_from_kv_strings("String".to_owned());
  /// assert_eq!(simple_obj.to_string(),"{\"Key\":\"Value\"}".to_owned());
  /// ```
  pub fn new_object_from_kv_strings(input_key: String, input_value: String) -> JsonData {
    JsonData {
      data: Rc::new(
              JsonDataType::new_object_from_kv_strings(input_key,input_value)
            ),
      parent: RefCell::new(Weak::new()),
    }
  }
  /// `array_wrap` wraps a JsonData into JsonDataType Array
  pub fn array_wrap(&self) -> Rc<JsonDataType> {
    Rc::new(
      JsonDataType::Array(
        RefCell::new(
          vec![
            JsonData {
              data: Rc::clone(&self.data),
              parent: RefCell::new(Weak::new()),
            }
          ]
        )
      )
    )
  }
  /// `obj_wrap` wraps a JsonData into JsonDataType Array
  pub fn obj_wrap(&self, key: String) -> Rc<JsonDataType> {
    Rc::new(
      JsonDataType::Object(
        RefCell::new(
          vec![
            (key,
              JsonData {
                data: Rc::clone(&self.data),
                parent: RefCell::new(Weak::new()),
              }
            )
          ]
        )
      )
    )
  }
  /// `to_string` returns the string representation of a JsonDataType object.
  pub fn to_string(&self) -> String {
    match &*self.data {
      JsonDataType::Atomic(data) => format!("\"{}\"", *data.borrow()),
      JsonDataType::Array(array_data) => {
        format!("[{}]",
                array_data
                .borrow()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(","))
      }
      JsonDataType::Object(obj_data) => {
        format!("{{{}}}",
                obj_data
                .borrow()
                .iter()
                .map(|(k,v)| format!("\"{}\":{}",k,v.to_string()))
                .collect::<Vec<_>>().join(","))
      },
      JsonDataType::Empty => "null".to_owned(),
    }
  }
  /// `from_xml_string` returns a JSON version of an XML string.
  /// If the XML is not valid returns Err.
  pub fn from_xml_string(data: &String, worker_id_string: &String, verbosity: i8) -> Result<JsonData,String> {
    let mut inside_tag = false;
    let mut cur_tag = String::with_capacity(128);
    let mut cur_tag_content = String::with_capacity(128);
    let mut cur_hierarchy:Vec<String> = vec![];
    let mut res = JsonData::new();
    if verbosity > 1 {
      println!("WORKER[{}] Working on content: {}", worker_id_string, data);
    }
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
          cur_tag.drain(..1); // Remove the leading /
          println!("WORKER[{}] Current val '{{{}:{}}}'", worker_id_string, cur_hierarchy.join("."), cur_tag_content);
          let cur_hier_len = cur_hierarchy.len() - 1;
          cur_hierarchy.remove(cur_hier_len);
          if cur_tag_content.len() > 0 {
            res.insert_kv(cur_tag.clone(),cur_tag_content.clone());
          } else {
            res.insert(cur_tag.clone());
          }
        } else {
          cur_hierarchy.push(cur_tag.clone());
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
    Ok(res)
  }
  /// `transform_to_atomic` an entry currently Empty variant to an Atomic variant.
  pub fn transform_to_atomic(&self, input: String) -> Result<Rc<JsonDataType>,String> {
    match &*self.data {
      JsonDataType::Empty => {
        Ok(Rc::new(JsonDataType::new_atomic_from_string(input)))
      },
      _ => {
        Err("transform_to_atomic supports only Empty JsonDataType".to_owned())
      },
    }
  }
  /// `push_value` to the current hierarchy.
  pub fn push_value(&mut self, input: String) {
    match &*self.data {
      JsonDataType::Array(array_ref) => {
        array_ref.borrow_mut().push(JsonData::new_atomic_from_string(input));
      },
//      JsonDataType::Object(obj_ref) => {
//        let obj_data = obj_ref.into_inner();
//        //obj_data.push((input, JsonData{ data: Rc::new(JsonDataType::Empty)}));
//        let mut inner_obj = obj_data[obj_data.len()-1].1.into_inner();
//        inner_obj.insert(input);
//      },
      _ => {},
    }
  }
  /// `vivify_path` Creates a path structure if needed and inserts a string
  /// on it.
  /// The path comes as an array of Strings to populate.
  /// If a step in path doesn't exist, it will be created as a JSON Object.
  /// If a step in path exists, it may need to be wrapped in a JSON Array.
  /// # Example:
  /// //```
  /// pub mod json;
  /// extern crate json;
  /// let mut test_obj = JsonData::new();
  /// test_obj.vivify_path("root.keys.alpha.id".to_owned(),"1".to_owned());
  /// assert_eq!(test_obj.to_string(),"{\"root\":{\"keys\":{\"alpha\":\"1\"}}}".to_owned());
  /// test_obj.vivify_path(&"root.keys.beta.id".split("."),"2".to_owned());
  /// assert_eq!(test_obj.to_string(),"{\"root\":{\"keys\":[{\"alpha\":\"1\"},{\"beta\":\"2\"}]}}".to_owned());
  /// test_obj.vivify_path("root.attrs.beta.name".to_owned(),"BetaAttr".to_owned());
  /// assert_eq!(test_obj.to_string(),"{\"root\":{\"keys\":[{\"alpha\":\"1\"},{\"beta\":\"2\"}]}}".to_owned());
  /// //```
  pub fn vivify_path(&mut self, path: &Vec<String>, input: String) {
    let new_item = JsonData::new_atomic_from_string(input);
    if let JsonDataType::Empty = *self.data {
        new_item.obj_wrap(path[0].clone());
    }
    let mut target = self;
    for step in path {
      let current_type = match &*self.data {
        JsonDataType::Object(kv_array_ref) => {
          let mut path_key = None;
          let kv_array = kv_array_ref.borrow_mut();
          for (k,v) in kv_array.iter() {
            if k == step {
              path_key = Some(v);
            }
          }
          kv_array.push((input_key, JsonData::new_atomic_from_string(input_value)));
          Rc::clone(&self.data)
          path_key
        },
        _ => None,
      };
      if Some(t) = path_key {
        target = t;
      } else {
        target.data = 
      }
    }
    *target.data = *new_item.data;
  }
  /// `insert` a string to the current level
  pub fn insert(&mut self, input: String) {
    match &*self.data {
      JsonDataType::Object(_) => {
        self.data = self.array_wrap();
        self.push_value(input);
        //    res.data = res.obj_wrap(cur_tag.clone());
        //panic!("Unsupported type for insert, use insert_kv");
      },
      JsonDataType::Array(_) => {
        self.push_value(input);
      },
      JsonDataType::Atomic(_) => {
        // XXX: How does one decide if something is to be transformed to Array or to Object?
        self.data = self.array_wrap();
        self.push_value(input);
      },
      JsonDataType::Empty => {
        self.data = self.transform_to_atomic(input).unwrap();
      },
    };
  }
  /// `insert_kv` Inserts a Key/Value pair.
  pub fn insert_kv(&mut self, input_key: String, input_value: String) {
    // If the key exists, add the input_value to the key.
    let new_data = match &*self.data {
      JsonDataType::Empty => {
        Rc::new(JsonDataType::new_object_from_kv_strings(input_key, input_value))
      },
      JsonDataType::Object(kv_array_ref) => {
        let mut kv_array = kv_array_ref.borrow_mut();
        for (k,v) in kv_array.iter_mut() {
          if *k == input_key {
            v.insert(input_value);
            return;
          }
        }
        kv_array.push((input_key, JsonData::new_atomic_from_string(input_value)));
        Rc::clone(&self.data)
      },
      _ => {
        println!("Unsupported type for insert_kv, use insert");
        Rc::clone(&self.data)
      },
    };
    self.data = new_data;
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_transforms_from_xml_string() {
    let verbosity = 3i8;
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
      "{\"body\":{\"div\":[{\"li\":\"1\"},{\"li\":\"2\"}]}}".to_owned()
    );
    assert_eq!(
      JsonData::from_xml_string(&"<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned(),&"TEST".to_owned(),verbosity).unwrap().to_string(),
      "{\"div\":{\"li\":[\"1\",\"2\"],\"lt\":\"a\"}}".to_owned()
    );
  }
  #[test]
  fn it_transforms_to_string() {
    // Technically this could be invalid JSON but not for our simple case.
    let simple_string = JsonData::new_atomic_from_string("String".to_owned());
    assert_eq!(simple_string.to_string(),"\"String\"".to_owned());
    let simple_array1 = JsonData::new_array_from_string("String".to_owned());
    assert_eq!(simple_array1.to_string(),"[\"String\"]".to_owned());
    let simple_array2 = JsonData{
      data: Rc::new(
        JsonDataType::Array(
          RefCell::new(
            vec![
              JsonData::new_atomic_from_string("String1".to_owned()),
              JsonData::new_atomic_from_string("String2".to_owned()),
            ]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    assert_eq!(simple_array2.to_string(),"[\"String1\",\"String2\"]".to_owned());
    let simple_obj = JsonData{
      data: Rc::new(
        JsonDataType::Object(
          RefCell::new(
            vec![("Key".to_owned(), JsonData::new_atomic_from_string("Value".to_owned()))]
          ),
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    assert_eq!(simple_obj.to_string(),"{\"Key\":\"Value\"}".to_owned());
    let multi_kv = JsonData{
      data: Rc::new(
        JsonDataType::Object(
          RefCell::new(
            vec![
              ("Key1".to_owned(), JsonData::new_atomic_from_string("Value1".to_owned())),
              ("Key2".to_owned(), JsonData::new_atomic_from_string("Value2".to_owned())),
            ]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":\"Value2\"}".to_owned());
    let array_contains_objs = JsonData{
      data: Rc::new(
        JsonDataType::Array(
          RefCell::new(
            vec![
              JsonData{
                data: Rc::new(
                  JsonDataType::Object(
                    RefCell::new(
                      vec![("Key".to_owned(), JsonData::new_atomic_from_string("Value1".to_owned()))]
                    ),
                  )
                ),
                parent: RefCell::new(Weak::new()),
              },
              JsonData{
                data: Rc::new(
                  JsonDataType::Object(
                    RefCell::new(
                      vec![("Key".to_owned(), JsonData::new_atomic_from_string("Value2".to_owned()))]
                    ),
                  )
                ),
                parent: RefCell::new(Weak::new()),
              },
            ]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"}]".to_owned());
    let obj_contains_array = JsonData{
      data: Rc::new(
        JsonDataType::Object(
          RefCell::new(
            vec![("Key".to_owned(), JsonData{
              data: Rc::new(
                JsonDataType::Array(
                  RefCell::new(
                    vec![
                      JsonData::new_atomic_from_string("Value1".to_owned()),
                      JsonData::new_atomic_from_string("Value2".to_owned()),
                    ]
                  )
                ),
              ),
              parent: RefCell::new(Weak::new()),
            })]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
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
    let mut simple_obj = JsonData::new_object_from_kv_strings("Key".to_owned(),"Value1".to_owned());
    simple_obj.insert_kv("Key".to_owned(),"Value2".to_owned());
    assert_eq!(simple_obj.to_string(),"{\"Key\":[\"Value1\",\"Value2\"]}".to_owned());
    let mut multi_kv = JsonData{ data: Rc::new(
      JsonDataType::Object(
        RefCell::new(
          vec![
            ("Key1".to_owned(), JsonData::new_atomic_from_string("Value1".to_owned())),
            ("Key2".to_owned(), JsonData::new_atomic_from_string("Value2.0".to_owned())),
          ]
        )
      )),
      parent: RefCell::new(Weak::new()),
    };
    multi_kv.insert_kv("Key2".to_owned(),"Value2.1".to_owned());
    assert_eq!(multi_kv.to_string(),"{\"Key1\":\"Value1\",\"Key2\":[\"Value2.0\",\"Value2.1\"]}".to_owned());
    let mut array_contains_objs = JsonData{
      data: Rc::new(
        JsonDataType::Array(
          RefCell::new(
            vec![
              JsonData{
                data: Rc::new(
                  JsonDataType::Object(
                    RefCell::new(
                      vec![("Key".to_owned(), JsonData::new_atomic_from_string("Value1".to_owned()))]
                    )
                  )
                ),
                parent: RefCell::new(Weak::new()),
              },
              JsonData{
                data: Rc::new(
                  JsonDataType::Object(
                    RefCell::new(
                      vec![("Key".to_owned(), JsonData::new_atomic_from_string("Value2".to_owned()))]
                    )
                  )
                ),
                parent: RefCell::new(Weak::new()),
              },
            ]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    array_contains_objs.insert("Value".to_owned());
    assert_eq!(array_contains_objs.to_string(),"[{\"Key\":\"Value1\"},{\"Key\":\"Value2\"},\"Value\"]".to_owned());
    let mut obj_contains_array = JsonData{
      data: Rc::new(
        JsonDataType::Object(
          RefCell::new(
            vec![("Key".to_owned(), JsonData{
              data: Rc::new(
                JsonDataType::Array(
                  RefCell::new(
                    vec![
                      JsonData::new_atomic_from_string("Value1".to_owned()),
                      JsonData::new_atomic_from_string("Value2".to_owned()),
                    ]
                  )
                )
              ),
              parent: RefCell::new(Weak::new()),
            })]
          )
        )
      ),
      parent: RefCell::new(Weak::new()),
    };
    obj_contains_array.insert_kv("Key".to_owned(),"Value3".to_owned());
    assert_eq!(obj_contains_array.to_string(),"{\"Key\":[\"Value1\",\"Value2\",\"Value3\"]}".to_owned());
  }
  #[test]
  fn it_vivifies_path() {
    let mut simple_obj = JsonData::new();
    let simple_path = vec!["Root".to_owned()];
    simple_obj.vivify_path(&simple_path, "Elem0".to_owned());
    assert_eq!(simple_obj.to_string(),"{\"Root\":\"Elem0\"}".to_owned());
  }
}
