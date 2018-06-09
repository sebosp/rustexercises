//! # Gain
//! A state machine whose output is always k times its input
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Gain<T>
where T: Num + Display + Clone + Copy
{
  pub k: T,
}
impl<T> super::StateMachine for Gain<T>
where T: Num + Display + Clone + Copy
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = T;
  /// `OutputType`(O) = numbers
  type OutputType = T;
  /// `K`(_s0_) = does not exist, K is defined at instantiation time.
  fn new(initial_value: Self::StateType) -> Self {
    Gain {
      k: initial_value
    }
  }
  fn start(&mut self){}
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(T::zero(),*inp)?;
    Ok(outp.1)
  }
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp * self.k)
  }
  fn get_next_values(&self, unused: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(unused,inp)?;
    Ok((next_state,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Gain K: {}",self.k)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.k)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = Gain::new(0f64);
    assert_eq!(test.get_next_values(0f64,0f64),Ok((0f64,0f64)));
    assert_eq!(test.get_next_values(0f64,0f64),Ok((0f64,0f64)));
  }
  #[test]
  fn it_steps() {
    let mut test = Gain::new(1f64);
    assert_eq!(test.step(&1f64),Ok(1f64));
    assert_eq!(test.step(&1f64),Ok(1f64));
    assert_eq!(test.k,1f64);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Gain::new(0f64);
    assert_eq!(test.get_next_state(1f64,1f64),Ok(0f64));
    let test2 = Gain::new(5f64);
    assert_eq!(test2.get_next_state(1f64,1f64),Ok(5f64));
    assert_eq!(test2.get_next_state(1f64,2f64),Ok(10f64));
  }
}
