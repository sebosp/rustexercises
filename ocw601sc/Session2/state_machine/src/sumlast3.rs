//! # SumLast3
//! state machine where the state is actually a list of values. State can be
//! anything (a dictionary, an array, a list, tuple); it is important to be sure
//! that the `get_next_values` method does not make direct changes to components
//! of the state, instead returning a new copy of the state with appropriate changes.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct SumLast3<T>
where T: Num + Display + Clone + Copy
{
  pub state: (T,T),
}
impl<T> super::StateMachine for SumLast3<T>
where T: Num + Display + Clone + Copy
{
  /// `StateType`(S) = tuple of numbers
  type StateType = (T,T);
  /// `InputType`(I) = number
  type InputType = T;
  /// `OutputType`(O) = number
  type OutputType = T;
  /// `initial_value`(_s0_) is (0,0)
  fn new(initial_value: Self::StateType) -> Self {
    SumLast3 {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = (T::zero(),T::zero())
  }
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok((state.1, *inp))
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state, Some(state.0 + next_state.0 + next_state.1)))
      }
    }
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,Some(inp))?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("SumLast3::Start state: ({},{})",self.state.0, self.state.1)
  }
  fn verbose_step(&self, inp: &Self::InputType, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("SumLast3::In: {} Out: None Next State: ({},{})", inp, self.state.0, self.state.1),
      Some(outp) => format!("SumLast3::In: {} Out: {} Next State: ({},{})", inp, outp, self.state.0, self.state.1)
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = SumLast3::new((0i8,1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&1i8),((0i8,1i8),1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(5i8,7i8),&3i8),((7i8,3i8),15i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,1i8),&5i8),((1i8,5i8),9i8));
  }
  #[test]
  fn it_steps() {
    let mut test = SumLast3::new((0i8,1i8));
    assert_eq!(test.step_unwrap(&2i8),3i8);
    assert_eq!(test.state,(1i8,2i8));
  }
  #[test]
  fn it_gets_next_state() {
    let test = SumLast3::new((0i8,0i8));
    assert_eq!(test.get_next_state(&(0i8,0i8),&1i8),Ok((0i8,1i8)));
    assert_eq!(test.get_next_state(&(1i8,3i8),&5i8),Ok((3i8,5i8)));
    assert_eq!(test.get_next_state(&(5i8,3i8),&1i8),Ok((3i8,1i8)));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = SumLast3::new((0i8,0i8));
    assert_eq!(test.is_composite(),false);
  }
}
