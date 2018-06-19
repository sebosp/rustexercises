//! # UpDown
//! This machine can count up and down; its state space is the countably
//! infinite set of integers. It starts in state 0. Now, if it gets input u,
//! it goes to state 1; if it gets u again, it goes to state 2. If it
//! gets d, it goes back down to 1, and so on. For this machine, the output is
//! always the same as the next state. 
//! This machine only supports 'u' or 'd' chars, returns Err(e) otherwise.
pub struct UpDown {
  pub state: i64,
}
impl super::StateMachine for UpDown {
  /// `StateType`(S) = number
  type StateType = i64;
  /// `InputType`(I) = char
  type InputType = char;
  /// `OutputType`(O) = number
  type OutputType = i64;
  /// `initial_value`(_s0_) is usually zero
  fn new(initial_value: Self::StateType) -> Self {
    UpDown {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = Self::StateType::from(0);
  }
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    let inp = *inp;
    let state = *state;
    if inp == 'u' {
      Ok(state + Self::StateType::from(1))
    } else  if inp == 'd' {
      Ok(state - Self::StateType::from(1))
    } else {
      Err("Invalid char for UpDown".to_string())
    }
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
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("State: {}",self.state)
  }
  fn state_machine_name(&self) -> String {
    "UpDown".to_string()
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
    let test = UpDown::new(0i64);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i64,&'d'),(-1i64,-1i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i64,&'u'),(1i64,1i64));
  }
  #[test]
  fn it_steps() {
    let mut test = UpDown::new(0i64);
    assert_eq!(test.step_unwrap(&'d'),-1i64);
    assert_eq!(test.state,(-1i64));
  }
  #[test]
  fn it_gets_next_state() {
    let test = UpDown::new(0i64);
    assert_eq!(test.get_next_state(&0i64,&'d'),Ok(-1i64));
    assert_eq!(test.get_next_state(&-1i64,&'u'),Ok(0i64));
    assert_eq!(test.get_next_state(&0i64,&'u'),Ok(1i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = UpDown::new(0i64);
    assert_eq!(test.is_composite(),false);
  }
}
