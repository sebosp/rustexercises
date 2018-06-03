//! # Average2
//! A machine whose output is the average of the current input and the
//! previous input. It stores its previous input as its state. 
//! State and Input must be i32 to allow for f64 conversion without
//! dropping of precision.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Average2<T>
where T: Num + Display + Clone + Copy + FromPrimitive
{
  pub state: T,
}
impl<T> super::StateMachine for Average2<T>
where T: Num + Display + Clone + Copy + FromPrimitive
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = T;
  /// `OutputType`(O) = numbers
  type OutputType = T;
  fn new(initial_value: Self::StateType) -> Self {
    Average2 {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = T::zero()
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    let cast_from_i8 = FromPrimitive::from_i8(2i8);
    match cast_from_i8 {
      Some(t) => Ok((next_state,Self::OutputType::from(state + next_state)/t)),
      None => Err("Unable to cast from i8".to_string()),
    }
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
