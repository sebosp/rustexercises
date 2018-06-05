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
      state: initial_value
    }
  }
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp + state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = Accumulator::new(0);
    assert_eq!(test.get_next_values(0i8,0i8),Ok((0i8,0i8)));
    assert_eq!(test.get_next_values(0i8,1i8),Ok((1i8,1i8)));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Accumulator::new(0);
    assert_eq!(test.get_next_state(0i8,0i8),Ok(0i8));
    assert_eq!(test.get_next_state(0i8,1i8),Ok(1i8));
  }
  #[test]
  fn it_steps_seq() {
    let mut test = Accumulator::new(0);
    assert_eq!(test.step(&1i8),Ok(1i8));
    assert_eq!(test.step(&2i8),Ok(3i8));
    assert_eq!(test.state,3i8);
  }
}
