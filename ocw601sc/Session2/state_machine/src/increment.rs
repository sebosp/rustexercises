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
  fn get_next_state(&self, state: &Self::StateType, _: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*state)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state,Some(next_state + *inp)))
      }
    }
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.incr,Some(inp))?;
    match outp.1 {
      None           => Ok(T::zero()),
      Some(next_val) => Ok(next_val),
    }
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
  fn it_gets_next_values_some() {
    let test = Increment::new(0f64);
    assert_eq!(test.get_next_values(&0f64,Some(&0f64)),Ok((0f64,Some(0f64))));
    assert_eq!(test.get_next_values(&0f64,Some(&7f64)),Ok((0f64,Some(7f64))));
    assert_eq!(test.get_next_values(&7f64,Some(&7f64)),Ok((7f64,Some(14f64))));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Increment::new(0f64);
    assert_eq!(test.get_next_values(&0f64,None),Ok((0f64,None)));
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
    assert_eq!(test.get_next_state(&1i64,&1i64),Ok(1i64));
    assert_eq!(test.get_next_state(&5i64,&7i64),Ok(5i64));
  }
}
