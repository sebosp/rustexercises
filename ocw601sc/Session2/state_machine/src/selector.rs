//! # Selector
//! A simple functional machine that is very useful is the Select machine. You
//! can make many different versions of this, but the simplest one takes an 
//! input that is a stream of lists or tuples of several values (or structures
//! of values) and generates the stream made up only of the kth elements of
//! the input values. 
use std::fmt::Debug; // XXX: Could this turn into a Display for usize?
#[derive(Debug)]
pub struct Selector<T>
where T: Debug + Clone + Copy
{
  pub k: usize,
  pub elems: Vec<T>
}
impl<T> super::StateMachine for Selector<T>
where T: Debug + Clone + Copy
{
  /// `StateType`(S) = slice size to return
  type StateType = usize;
  /// `InputType`(I) = Vector of numbers
  type InputType = Vec<T>;
  /// `OutputType`(O) = Vector of numbers
  type OutputType = Vec<T>;
  /// `num_elements`(k)
  fn new(num_elements: Self::StateType) -> Self {
    Selector {
      k: num_elements,
      elems: Vec::new()
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    if self.k > inp.len() {
      Err("Requested index out of bounds".to_string())
    } else {
      Ok(self.k)
    }
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        // Might be expensive to clone the Vector if it's big.
        let next_state = self.get_next_state(state,inp)?;
        let mut res: Self::OutputType = inp.to_vec();
        res.truncate(self.k);
        Ok((next_state,Some(res)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: i8) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.k,inp)?;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("K: ({:?})",self.k)
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("In: None"),
      Some(outp) => format!("In: {:?}", outp),
    }
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None      => format!("In: None"),
      Some(inp) => format!("In: {:?}", inp),
    }
  }
  fn verbose_step(&self, inp: Option<&Self::InputType>, outp: Option<&Self::OutputType>) -> String {
    format!("{}: {} {} {}", self.state_machine_name(), self.verbose_input(inp),self.verbose_output(outp), self.verbose_state())
  }
  fn state_machine_name(&self) -> String {
    "Selector".to_string()
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_good() {
    let test1 = Selector::new(0usize);
    assert_eq!(test1.get_next_values_wrap_unwrap(&0usize,&vec!['a','b']),(0usize,vec![]));
    let test2 = Selector::new(1usize);
    assert_eq!(test2.get_next_values_wrap_unwrap(&1usize,&vec!['a','b']),(1usize,vec!['a']));
    let test3 = Selector::new(2usize);
    assert_eq!(test3.get_next_values_wrap_unwrap(&1usize,&vec!['a','b']),(2usize,vec!['a','b']));
  }
  #[test]
  fn it_gets_next_values_bad_range() {
    let test = Selector::new(100usize);
    assert_eq!(test.get_next_values(&3usize,Some(&vec!['a','b'])),Err("Requested index out of bounds".to_string()));
  }
  #[test]
  fn it_steps() {
    let mut test = Selector::new(0);
    assert_eq!(test.step_unwrap(&vec!['a','b']),vec![]);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Selector::new(1usize);
    assert_eq!(test.get_next_state(&1usize,&vec!['a']),Ok(1usize));
    assert_eq!(test.get_next_state(&2usize,&vec!['a']),Ok(1usize));
    assert_eq!(test.get_next_state(&3usize,&vec!['a']),Ok(1usize));
    assert_eq!(test.get_next_state(&4usize,&vec!['a']),Ok(1usize));
  }
  #[test]
  fn it_checks_is_composite() {
    let test: Selector<usize> = Selector::new(1usize);
    assert_eq!(test.is_composite(),false);
  }
}
