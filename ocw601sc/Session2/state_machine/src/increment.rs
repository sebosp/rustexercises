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
      incr: initial_value,
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
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.incr,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.incr),
             self.verbose_input(inp),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref()))
    }
    Ok(outp.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("Increment: {}", state)
  }
  fn state_machine_name(&self) -> String {
    "Increment".to_string()
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
  fn get_state(&self) -> Self::StateType{
    self.incr
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_some() {
    let test = Increment::new(0f64);
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&0f64),(0f64,0f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&7f64),(0f64,7f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&7f64,&7f64),(7f64,14f64));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Increment::new(0f64);
    assert_eq!(test.get_next_values(&0f64,None),Ok((0f64,None)));
  }
  #[test]
  fn it_steps() {
    let mut test = Increment::new(1f64);
    assert_eq!(test.step_unwrap(&1f64),2f64);
    assert_eq!(test.step_unwrap(&1f64),2f64);
    assert_eq!(test.incr,1f64);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Increment::new(0i64);
    assert_eq!(test.get_next_state(&1i64,&1i64),Ok(1i64));
    assert_eq!(test.get_next_state(&5i64,&7i64),Ok(5i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Increment::new(0i64);
    assert_eq!(test.is_composite(),false);
  }
}
