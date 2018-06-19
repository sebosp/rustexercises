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
      k: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*inp * self.k)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state,Some(next_state)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&T::zero(),inp)?;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("K: {}",self.k)
  }
  fn state_machine_name(&self) -> String {
    "Gain".to_string()
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
  fn verbose_step(&self, inp: Option<&Self::InputType>, outp: Option<&Self::OutputType>) -> String {
    format!("{}: {} {} {}", self.state_machine_name(), self.verbose_input(inp),self.verbose_output(outp), self.verbose_state())
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values() {
    let test = Gain::new(0f64);
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&0f64),(0f64,0f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&0f64),(0f64,0f64));
  }
  #[test]
  fn it_steps() {
    let mut test = Gain::new(1f64);
    assert_eq!(test.step_unwrap(&1f64),1f64);
    assert_eq!(test.step_unwrap(&1f64),1f64);
    assert_eq!(test.k,1f64);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Gain::new(0f64);
    assert_eq!(test.get_next_state(&1f64,&1f64),Ok(0f64));
    let test2 = Gain::new(5f64);
    assert_eq!(test2.get_next_state(&1f64,&1f64),Ok(5f64));
    assert_eq!(test2.get_next_state(&1f64,&2f64),Ok(10f64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Gain::new(0f64);
    assert_eq!(test.is_composite(),false);
  }
}
