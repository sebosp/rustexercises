//! # XML To JSON
//! This is a State Machine that transforms from XML to JSON.
extern crate treexml;
extern crate serde_json;
extern crate node2object;
pub struct XmlToJson {
  pub state: bool,
}
impl super::StateMachine for XmlToJson {
  /// `StateType`(S) Nothing in particular for state.
  type StateType = bool;
  /// `InputType`(I) A String, the JSON request object.
  type InputType = String;
  /// `OutputType`(O) A String, the JSON response object.
  type OutputType = String;
  /// Returns an ABC struct. `initial_value`(_s0_) is usually 0.
  fn new(initial_value: Self::StateType) -> Self {
    XmlToJson {
      state: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, _: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(true)
  }
  /// XmlToJson uses the input string and returns a JSON representation of it.
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None      => Ok((*state,None)),
      Some(inp) => {
        let tree = treexml::Document::parse(inp.as_bytes());
        match tree {
          Ok(v) => {
            let dom_root = v.root.unwrap();
            let json_rep = serde_json::Value::Object(node2object::node2object(&dom_root));
            Ok((true,Some(json_rep.to_string())))
          },
          Err(e) => {
            Err(e.description().to_string())
          }
        }
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref()))
    }
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self, _state: &Self::StateType) -> String {
    format!("No State.")
  }
  fn state_machine_name(&self) -> String {
    "XmlToJson".to_string()
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  => format!("In: {}", inp),
    }
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: None"),
      Some(outp) => format!("Out: {}", outp),
    }
  }
  fn get_state(&self) -> Self::StateType{
    self.state
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_good_xml() {
    let test = XmlToJson::new(true);
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<li>1</li>".to_owned())),
      Ok((true,Some("{\"li\":1.0}".to_owned())))
    );
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<div><li>1</li></div>".to_owned())),
      Ok((true,Some("{\"div\":{\"li\":1.0}}".to_owned())))
    );
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<div><li>1</li><li>2</li></div>".to_owned())),
      Ok((true,Some("{\"div\":{\"li\":[1.0,2.0]}}".to_owned())))
    );
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<body><div><li>1</li></div><div><li>2</li></div></body>".to_owned())),
      Ok((true,Some("{\"body\":{\"div\":[{\"li\":1.0},{\"li\":2.0}]}}".to_owned())))
    );
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<div><li>1</li><lt>a</lt><li>2</li></div>".to_owned())),
      Ok((true,Some("{\"div\":{\"li\":[1.0,2.0],\"lt\":\"a\"}}".to_owned())))
    );
  }
  #[test]
  fn it_gets_next_values_bad_xml() {
    let test = XmlToJson::new(true);
    assert_eq!(test.get_next_values(
      &true,
      Some(&"<div".to_owned())),
      Err("Unexpected end of stream".to_owned())
    );
  }
  #[test]
  fn it_checks_is_composite() {
    let test = XmlToJson::new(false);
    assert_eq!(test.is_composite(),false);
  }
}
