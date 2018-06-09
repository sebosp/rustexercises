//! # Delay
//! A machine that delays its input stream by one time step, we have to specify
//! what the first output should be (`initial_value`).
//! The state of a Delay machine is just the input from the previous step, and
//! the output is the state (which is, therefore, the input from the previous
//! time step). The Delay struct is also knows as *_R_*
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Delay<T>
where T: Num + Display + Clone + Copy
{
  pub state: T,
}
impl<T> super::StateMachine for Delay<T>
where T: Num + Display + Clone + Copy
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = T;
  /// `OutputType`(O) = numbers
  type OutputType = T;
  /// `initial_value`(_s0_) is defined when initialized.
  fn new(initial_value: Self::StateType) -> Self {
    Delay {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = T::zero();
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn get_next_state(&self, state: Self::StateType, _: Self::InputType) -> Result<Self::StateType, String> {
    Ok(state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((inp,next_state))
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
    let test = Delay::new(0);
    assert_eq!(test.get_next_values(0i8,1i8),Ok((1i8,0i8)));
    assert_eq!(test.get_next_values(2i8,3i8),Ok((3i8,2i8)));
    assert_eq!(test.get_next_values(4i8,5i8),Ok((5i8,4i8)));
  }
  #[test]
  fn it_steps() {
    let mut test = Delay::new(0);
    assert_eq!(test.step(&1i8),Ok(0i8));
    assert_eq!(test.state,1i8);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Delay::new(0);
    assert_eq!(test.get_next_state(0i8,1i8),Ok(0i8));
    assert_eq!(test.get_next_state(1i8,0i8),Ok(1i8));
  }
}
