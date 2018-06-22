//! # Wire
//! A Simple StateMachine who's Output is just the Input
use std::fmt::Display;
pub struct Wire<T>
where T: Display + Clone + Copy
{
  pub state: T,
}
impl<T> super::StateMachine for Wire<T>
where T: Display + Clone + Copy
{
  /// `StateType`(S) = T
  type StateType = T;
  /// `InputType`(I) = T
  type InputType = T;
  /// `OutputType`(O) = T
  type OutputType = T;
  /// `initial_value`(_s0_) is defined when initialized.
  fn new(initial_value: Self::StateType) -> Self {
    Wire {
      state: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*inp)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,Some(*state))),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((*inp,Some(next_state)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("State: {}",self.state)
  }
  fn state_machine_name(&self) -> String {
    "Wire".to_string()
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
  fn it_gets_next_values_some() {
    let test = Wire::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&1i8),(1i8,1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&2i8,&3i8),(3i8,3i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&4i8,&5i8),(5i8,5i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Wire::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
  }
  #[test]
  fn it_steps() {
    let mut test = Wire::new(0);
    assert_eq!(test.step_unwrap(&1i8),1i8);
    assert_eq!(test.state,1i8);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Wire::new(0);
    assert_eq!(test.get_next_state(&0i8,&1i8),Ok(1i8));
    assert_eq!(test.get_next_state(&1i8,&0i8),Ok(0i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Wire::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
