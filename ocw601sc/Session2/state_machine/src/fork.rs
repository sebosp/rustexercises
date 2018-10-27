//! # Fork Composite StateMachine
//! One input feeds two StateMachines, not to be confused with parallel
//! execution (yet), both StateMachine Input is the same type/value.
//! This should be a Parallel Trait that simply has InputType the same
//! for both state machines...
pub struct Fork<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: SM1,
  pub sm2: SM2,
  pub state: (<SM1>::StateType,<SM2>::StateType),
}
impl<SM1,SM2> super::StateMachine for Fork<SM1,SM2> 
  where SM1: super::StateMachine,
        SM2: super::StateMachine,
        SM1: super::StateMachine<InputType=<SM2>::InputType>,
        <SM1>::StateType: Clone + Copy,
        <SM2>::StateType: Clone + Copy,
        <SM1>::InputType: Clone + Copy,
        <SM2>::InputType: Clone + Copy,
        <SM1>::OutputType: PartialEq + Clone + Copy,
        <SM2>::OutputType: PartialEq + Clone + Copy,
{
  /// `StateType`(S) = numbers
  type StateType = (<SM1>::StateType,<SM2>::StateType);
  /// `InputType`(I) = numbers
  type InputType = <SM1>::InputType;
  /// `OutputType`(O) = numbers
  type OutputType = super::DualValues<<SM1>::OutputType,<SM2>::OutputType>;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Fork {
      sm1: <SM1>::new(initial_value.0),
      sm2: <SM2>::new(initial_value.1),
      state: initial_value,
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> 
  where SM1: super::StateMachine<InputType=<SM2>::InputType>,
  {
    let sm1_next_state = self.sm1.get_next_state(&state.0,inp)?;
    let sm2_next_state = self.sm2.get_next_state(&state.1,inp)?;
    Ok((sm1_next_state,sm2_next_state))
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> 
  where SM1: super::StateMachine<InputType=<SM2>::InputType>,
  {
    let sm1_next_values = self.sm1.get_next_values(&state.0,inp)?;
    let sm2_next_values = self.sm2.get_next_values(&state.1,inp)?;
    // Technically this could be just a None, instead of a Some(None,None), maybe worth it for a
    // future state machine.
    Ok(((sm1_next_values.0,sm2_next_values.0),Some(super::DualValues{ val1: sm1_next_values.1, val2: sm2_next_values.1})))
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp));
    }
    let _ = self.sm1.step(inp,verbose,depth+1)?;
    let _ = self.sm2.step(inp,verbose,depth+1)?;
    if verbose {
      println!("{}{}::{} {}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref()));
    }
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("[{}::{},{}::{}]",self.sm1.state_machine_name(),self.sm1.verbose_state(&state.0),self.sm2.state_machine_name(),self.sm2.verbose_state(&state.1))
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    self.sm1.verbose_input(inp)
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: (None)"),
      Some(outp) => format!("({},{})",self.sm1.verbose_output(outp.val1.as_ref()),self.sm2.verbose_output(outp.val2.as_ref()))
    }
  }
  fn state_machine_name(&self) -> String {
    "Fork".to_string()
  }
  fn is_composite(&self) -> bool {
    true
  }
  fn get_state(&self) -> Self::StateType{
    self.state
  }
}
pub struct ForkBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: Option<SM1>,
  pub sm2: Option<SM2>,
  pub state: (Option<<SM1>::StateType>,Option<<SM2>::StateType>),
}
impl<SM1,SM2> ForkBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine,
        SM1: super::StateMachine<InputType=<SM2>::InputType>,
        <SM1>::StateType: Clone + Copy,
        <SM2>::StateType: Clone + Copy,
        <SM1>::InputType: Clone + Copy,
        <SM2>::InputType: Clone + Copy,
        <SM1>::OutputType: PartialEq + Clone + Copy,
        <SM2>::OutputType: PartialEq + Clone + Copy,
{
  pub fn new() -> ForkBuilder<SM1,SM2> {
    ForkBuilder{
      sm1: None,
      sm2: None,
      state: (None,None),
    }
  }
  pub fn with_path1(mut self, input: SM1) -> ForkBuilder<SM1,SM2> {
    self.state.0 = Some(input.get_state());
    self.sm1 = Some(input);
    self
  }
  pub fn with_path2(mut self, input: SM2) -> ForkBuilder<SM1,SM2> {
    self.state.1 = Some(input.get_state());
    self.sm2 = Some(input);
    self
  }
  pub fn build(self) -> Result<Fork<SM1,SM2>,String> {
    let state1 = match self.state.0 {
      Some(val) => val,
      None => return Err("Missing initial state for 1st State Machine".to_owned()),
    };
    let state2 = match self.state.1 {
      Some(val) => val,
      None => return Err("Missing initial state for 2nd State Machine.".to_string()),
    };
    let sm1 = match self.sm1 {
      Some(val) => val,
      None => return Err("Missing 1st State Machine definition (with_path)".to_string()),
    };
    let sm2 = match self.sm2 {
      Some(val) => val,
      None => return Err("Missing 2nd State Machine Definition (with_path)".to_string()),
    };
    Ok(Fork{
      sm1: sm1,
      sm2: sm2,
      state: (state1, state2),
    })
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use accumulator::Accumulator;
  use increment::Increment;
  #[test]
  fn it_get_next_values_accumulators() {
    let test: Fork<Accumulator<i8>,Accumulator<i8>> = Fork::new((1i8,2i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&0i8),((0i8,0i8), DualValues{val1: Some(0i8), val2: Some(0i8)}));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,12i8), DualValues{val1: Some(10i8), val2: Some(12i8)}));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,12i8), DualValues{val1: Some(10i8), val2: Some(12i8)}));
  }
  #[test]
  fn it_get_next_state_accumulators() {
    let test: Fork<Accumulator<i8>,Accumulator<i8>> = Fork::new((1i8,2i8));
    assert_eq!(test.get_next_state(&(0i8,0i8),&0i8),Ok((0i8,0i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&7i8),Ok((10i8,12i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&7i8),Ok((10i8,12i8)));
  }
  #[test]
  fn it_steps_accumulators() {
    let mut test: Fork<Accumulator<i8>,Accumulator<i8>> = Fork::new((1i8,2i8));
    assert_eq!(test.step_unwrap(&3i8), DualValues{ val1: Some(4i8), val2: Some(5i8)});
    assert_eq!(test.state,(4i8,5i8));
    assert_eq!(test.step_unwrap(&5i8), DualValues{ val1: Some(9i8), val2: Some(10i8)});
    assert_eq!(test.state,(9i8,10i8));
  }
  #[test]
  fn it_steps_increments() {
    let mut test: Fork<Increment<i64>,Increment<i64>> = Fork::new((100i64,1i64));
    assert_eq!(test.step_unwrap(&3i64), DualValues{ val1: Some(103i64), val2: Some(4i64)});
    assert_eq!(test.state,(100i64,1i64));
    assert_eq!(test.step_unwrap(&2i64), DualValues{ val1: Some(102i64), val2: Some(3i64)});
    assert_eq!(test.state,(100i64,1i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test: Fork<Accumulator<i8>,Accumulator<i8>> = Fork::new((1i8,2i8));
    assert_eq!(test.is_composite(),true);
  }
}
