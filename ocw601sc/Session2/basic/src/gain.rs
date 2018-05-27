//! # Gain
//! A state machine whose output is always k times its input
pub struct Gain {
  pub k: i64,
}
impl super::StateMachine for Gain {
  /// `StateType`(S) = numbers
  type StateType = i64;
  /// `InputType`(I) = numbers
  type InputType = i64;
  /// `OutputType`(O) = numbers
  type OutputType = i64;
  /// `K`(_s0_) = does not exist, K is defined at instantiation time.
  fn new(initial_value: Self::StateType) -> Self {
    Gain {
      k: initial_value
    }
  }
  fn start(&mut self){}
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(Self::StateType::from(0),*inp)?;
    Ok(outp.1)
  }
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp * self.k)
  }
  fn get_next_values(&self, unused: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(unused,inp)?;
    Ok((next_state,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Gain K: {}",self.k)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.k)
  }
}
