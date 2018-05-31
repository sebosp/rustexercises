//! # Cascade StateMachine
//! The input of a StateMachine becomes the output of the second StateMachine.
pub struct Cascade<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: SM1,
  pub sm2: SM2,
  pub state: (<SM1>::StateType,<SM2>::StateType),
}
impl<SM1,SM2> super::StateMachine for Cascade<SM1,SM2> 
  where SM1: super::StateMachine,
        SM2: super::StateMachine,
        SM1: super::StateMachine<OutputType=<SM2>::InputType>,
        <SM1>::StateType: Clone + Copy,
        <SM2>::StateType: Clone + Copy,
        <SM1>::InputType: Clone + Copy,
{
  /// `StateType`(S) = numbers
  type StateType = (<SM1>::StateType,<SM2>::StateType);
  /// `InputType`(I) = numbers
  type InputType = <SM1>::InputType;
  /// `OutputType`(O) = numbers
  type OutputType = <SM2>::OutputType;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Cascade {
      sm1: <SM1>::new(initial_value.0),
      sm2: <SM2>::new(initial_value.1),
      state: initial_value,
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> 
  where SM1: super::StateMachine<OutputType=<SM2>::InputType>,
  {
    let sm1_next_value = self.sm1.get_next_values(state.0,inp)?;
    let sm2_next_state = self.sm2.get_next_state(state.1,sm1_next_value.1)?;
    Ok((sm1_next_value.0,sm2_next_state))
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String>
  where SM1: super::StateMachine<OutputType=<SM2>::InputType>,
  {
    let sm1_next_state = self.sm1.get_next_values(state.0,inp)?;
    let sm2_next_state = self.sm2.get_next_values(state.1,sm1_next_state.1)?;
    Ok(((sm1_next_state.0,sm2_next_state.0),sm2_next_state.1))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: (SM1:{}, SM2:{})",self.sm1.verbose_state(),self.sm2.verbose_state())
  }
  fn verbose_step(&self,_: &Self::InputType, _: &Self::OutputType) -> String {
     format!("(SM1:{}, SM2:{})", self.sm1.verbose_state(),self.sm2.verbose_state())
  }
}
