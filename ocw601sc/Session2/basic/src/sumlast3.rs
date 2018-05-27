//! # SumLast3
//! state machine where the state is actually a list of values. State can be
//! anything (a dictionary, an array, a list, tuple); it is important to be sure
//! that the `get_next_values` method does not make direct changes to components
//! of the state, instead returning a new copy of the state with appropriate changes.
pub struct SumLast3 {
  pub state: (i64,i64),
}
impl super::StateMachine for SumLast3 {
  /// `StateType`(S) = tuple of numbers
  type StateType = (i64,i64);
  /// `InputType`(I) = number
  type InputType = i64;
  /// `OutputType`(O) = number
  type OutputType = i64;
  /// `initial_value`(_s0_) is (0,0)
  fn new(initial_value: Self::StateType) -> Self {
    SumLast3 {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = (0i64,0i64)
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok((state.1, inp))
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state, state.0 + next_state.0 + next_state.1))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: ({},{})",self.state.0, self.state.1)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: ({},{})", inp, outp, self.state.0, self.state.1)
  }
}
