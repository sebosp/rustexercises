//! # Average2
//! A machine whose output is the average of the current input and the
//! previous input. It stores its previous input as its state. 
//! State and Input must be i32 to allow for f64 conversion without
//! dropping of precision.
pub struct Average2 {
  pub state: i32,
}
impl super::StateMachine for Average2 {
  /// `StateType`(S) = numbers
  type StateType = i32;
  /// `InputType`(I) = numbers
  type InputType = i32;
  /// `OutputType`(O) = numbers
  type OutputType = f64;
  fn new(initial_value: Self::StateType) -> Self {
    Average2 {
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
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,Self::OutputType::from(state + next_state)/2f64))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
