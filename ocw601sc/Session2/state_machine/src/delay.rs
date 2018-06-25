//! # Delay
//! A machine that delays its input stream by one time step, we have to specify
//! what the first output should be (`initial_value`).
//! The state of a Delay machine is just the input from the previous step, and
//! the output is the state (which is, therefore, the input from the previous
//! time step). The Delay struct is also knows as *_R_*
use std::fmt::Display;
pub struct Delay<T>
where T: Display + Clone + Copy
{
  pub state: T,
}
impl<T> super::StateMachine for Delay<T>
where T: Display + Clone + Copy
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
      state: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, state: &Self::StateType, _: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*state)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,Some(*state))), // When receiving None it should return the current value
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((*inp,Some(next_state)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref()))
    }
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}", state)
  }
  fn state_machine_name(&self) -> String {
    "Delay".to_string()
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
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_some() {
    let test = Delay::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&1i8),(1i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&2i8,&3i8),(3i8,2i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&4i8,&5i8),(5i8,4i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Delay::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
  }
  #[test]
  fn it_steps() {
    let mut test = Delay::new(0);
    assert_eq!(test.step_unwrap(&1i8),0i8);
    assert_eq!(test.state,1i8);
  }
  #[test]
  fn it_gets_next_state() {
    let test = Delay::new(0);
    assert_eq!(test.get_next_state(&0i8,&1i8),Ok(0i8));
    assert_eq!(test.get_next_state(&1i8,&0i8),Ok(1i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Delay::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
