//! # Negation
//! This machine can count up and down; its state space is the countably
//! infinite set of integers. It starts in state 0. Now, if it gets input u,
//! it goes to state 1; if it gets u again, it goes to state 2. If it
//! gets d, it goes back down to 1, and so on. For this machine, the output is
//! always the same as the next state. 
//! This machine only supports 'u' or 'd' chars, returns Err(e) otherwise.
pub struct Negation {
  pub state: bool,
}
impl super::StateMachine for Negation {
  /// `StateType`(S) = number
  type StateType = bool;
  /// `InputType`(I) = char
  type InputType = bool;
  /// `OutputType`(O) = number
  type OutputType = bool;
  /// `initial_value`(_s0_) is usually zero
  fn new(initial_value: Self::StateType) -> Self {
    Negation {
      state: initial_value
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(!inp)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,Some(*state))),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state,Some(next_state)))
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
    "Negation".to_string()
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
  fn it_gets_next_values() {
    let test = Negation::new(false);
    assert_eq!(test.get_next_values_wrap_unwrap(&false,&false),(true,true));
    assert_eq!(test.get_next_values_wrap_unwrap(&false,&true),(false,false));
    assert_eq!(test.get_next_values_wrap_unwrap(&true,&false),(true,true));
    assert_eq!(test.get_next_values_wrap_unwrap(&true,&true),(false,false));
  }
  #[test]
  fn it_steps() {
    let mut test = Negation::new(false);
    assert_eq!(test.step_unwrap(&false),true);
    assert_eq!(test.state,(true));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Negation::new(false);
    assert_eq!(test.get_next_state(&false,&false),Ok(true));
    assert_eq!(test.get_next_state(&false,&true),Ok(false));
    assert_eq!(test.get_next_state(&true,&false),Ok(true));
    assert_eq!(test.get_next_state(&true,&true),Ok(false));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Negation::new(true);
    assert_eq!(test.is_composite(),false);
  }
}
