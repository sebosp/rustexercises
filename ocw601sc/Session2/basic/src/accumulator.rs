//! # Accumulator
//! A machine whose output is the sum of all the inputs it has ever seen.
pub struct Accumulator {
  pub start_state: i64,
  pub state: i64,
}
impl super::StateMachine for Accumulator {
  /// `StateType`(S) = numbers
  type StateType = i64;
  /// `InputType`(I) = numbers
  type InputType = i64;
  /// `OutputType`(O) = numbers
  type OutputType = i64;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Accumulator {
      start_state: initial_value,
      state: initial_value
    }
  }
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp + state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
