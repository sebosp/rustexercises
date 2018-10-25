//! # Accumulator
//! A machine whose output is the sum of all the inputs it has ever seen.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Accumulator<T>
where T: Num + Display + Clone + Copy
{
  pub start_state: T,
  pub state: T,
}
impl<T> super::StateMachine for Accumulator<T>
where T: Num + Display + Clone + Copy
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = T;
  /// `OutputType`(O) = numbers
  type OutputType = T;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Accumulator {
      start_state: initial_value,
      state: initial_value,
    }
  }
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*inp + *state)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None      => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state,Some(next_state)))
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
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}", state)
  }
  fn state_machine_name(&self) -> String {
    "Accumulator".to_string()
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
  fn get_current_state(&self) -> Self::StateType{
    self.state
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = Accumulator::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&0i8),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&1i8),(1i8,1i8));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Accumulator::new(0);
    assert_eq!(test.get_next_state(&0i8,&0i8),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&1i8),Ok(1i8));
  }
  #[test]
  fn it_steps_seq() {
    let mut test = Accumulator::new(0);
    assert_eq!(test.step_unwrap(&1i8),1i8);
    assert_eq!(test.step_unwrap(&2i8),3i8);
    assert_eq!(test.state,3i8);
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Accumulator::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
