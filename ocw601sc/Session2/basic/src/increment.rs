//! # Increment
//! A machine whose output at time t is the input at time t plus a constant incr.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Increment<T>
where T: Num + Display + Clone + Copy
{
  pub incr: T,
}
impl<T> super::StateMachine for Increment<T>
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
    Increment {
      incr: initial_value
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp + state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.incr,*inp)?;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.incr)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.incr)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = Increment::new(0f64);
    assert_eq!(test.get_next_values(0f64,0f64),Ok((0f64,0f64)));
    assert_eq!(test.get_next_values(0f64,0f64),Ok((0f64,0f64)));
  }
  #[test]
  fn it_steps() {
    let mut test = Increment::new(1f64);
    assert_eq!(test.step(&1f64),Ok(2f64));
    assert_eq!(test.step(&1f64),Ok(2f64));
    assert_eq!(test.incr,1f64);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Increment::new(0i64);
    assert_eq!(test.get_next_state(1i64,1i64),Ok(2i64));
    assert_eq!(test.get_next_state(5i64,7i64),Ok(12i64));
  }
}
