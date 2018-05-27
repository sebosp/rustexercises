//! # Delay
//! A machine that delays its input stream by one time step, we have to specify
//! what the first output should be (`initial_value`).
//! The state of a Delay machine is just the input from the previous step, and
//! the output is the state (which is, therefore, the input from the previous
//! time step). The Delay struct is also knows as *_R_*
pub struct Delay {
  pub state: i64,
}
impl super::StateMachine for Delay {
  /// `StateType`(S) = numbers
  type StateType = i64;
  /// `InputType`(I) = numbers
  type InputType = i64;
  /// `OutputType`(O) = numbers
  type OutputType = i64;
  /// `initial_value`(_s0_) is defined when initialized.
  fn new(initial_value: Self::StateType) -> Self {
    Delay {
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
  fn get_next_state(&self, state: Self::StateType, _: Self::InputType) -> Result<Self::StateType, String> {
    Ok(state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((inp,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
