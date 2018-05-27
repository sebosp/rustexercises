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
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    if inp == 'u' {
      Ok(state + Self::StateType::from(1))
    } else  if inp == 'd' {
      Ok(state - Self::StateType::from(1))
    } else {
      Err("Invalid char for UpDown".to_string())
    }
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
