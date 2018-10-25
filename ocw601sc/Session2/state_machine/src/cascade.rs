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
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> 
  where SM1: super::StateMachine<OutputType=<SM2>::InputType>,
  {
    // In order to get the Output from StateMachine1, we need to go through get_next_values...
    let sm1_next_value = self.sm1.get_next_values(&state.0,Some(inp))?;
    let sm2_next_state = self.sm2.get_next_values(&state.1,sm1_next_value.1.as_ref())?;
    Ok((sm1_next_value.0,sm2_next_state.0))
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>
  where SM1: super::StateMachine<OutputType=<SM2>::InputType>,
  {
    let sm1_next_value = self.sm1.get_next_values(&state.0,inp)?;
    let sm2_next_value = self.sm2.get_next_values(&state.1,sm1_next_value.1.as_ref())?;
    Ok(((sm1_next_value.0,sm2_next_value.0),sm2_next_value.1))
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      //println!("{{\"Class\":\"{}\",[{{\"SM1\":{} }},{{\"SM2\":{} }}]}}", // XXX: JSON
      println!("{}{}::{} {}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp));
    }
    let sm1_outp = self.sm1.step(inp,verbose,depth+1)?;
    let _ = self.sm2.step(sm1_outp.as_ref(),verbose,depth+1)?;
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
    format!("({}::{},{}::{})",self.sm1.state_machine_name(),self.sm1.verbose_state(&state.0),self.sm2.state_machine_name(),self.sm2.verbose_state(&state.1))
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    self.sm1.verbose_input(inp)
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    self.sm2.verbose_output(outp)
  }
  fn state_machine_name(&self) -> String {
    "Cascade".to_string()
  }
  fn is_composite(&self) -> bool {
    true
  }
  fn get_current_state(&self) -> Self::StateType{
    self.state
  }
}
struct CascadeBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: Option<SM1>,
  pub sm2: Option<SM2>,
  pub state: (Option<<SM1>::StateType>,Option<<SM2>::StateType>),
}
impl<SM1,SM2> CascadeBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub fn new() -> CascadeBuilder<SM1,SM2> {
    CascadeBuilder{
      sm1: None,
      sm2: None,
      state: (None,None),
    }
  }
  pub fn with_src(mut self, input: SM1) -> CascadeBuilder<SM1,SM2> {
    self.sm1 = Some(input);
    self.state.0 = Some(sm1.get_current_state());
    self
  }
  pub fn with_dst(mut self, input: SM2) -> CascadeBuilder<SM1,SM2> {
    self.sm2 = Some(input);
    self.state.1 = Some(sm2.get_current_state);
    self
  }
  pub fn build(self) -> Result<Cascade<SM1,SM2>,String> {
    let src_state = match self.state.0 {
      Some(val) => val,
      None => return Err("Missing initial state for Source State Machine".to_owned()),
    };
    let dst_state = match self.state.1 {
      Some(val) => val,
      None => return Err("Missing initial state for Destination State Machine.".to_string()),
    };
    let src = match self.sm1 {
      Some(val) => val,
      None => return Err("Missing Source State Machine definition (with_src)".to_string()),
    };
    let dst = match self.sm2 {
      Some(val) => val,
      None => return Err("Missing Destination State Machine Definition (with_dst)".to_string()),
    };
    Ok(Cascade{
      sm1: src,
      sm2: dst,
      state: (src_state, dst_state),
    })
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use accumulator::Accumulator;
  use average2::Average2;
  use delay::Delay;
  use increment::Increment;
  use fork::Fork;
  use adder::Adder;
  use wire::Wire;
  #[test]
  fn it_builds_cascades() {
    let accum1 = Accumulator::new(1i8);
    let accum2 = Accumulator::new(2i8);
    let test = CascadeBuilder::new()
      .with_src(accum1)
      .with_dst(accum2)
      .build();
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&0i8),((0i8,0i8),0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,15i8),15i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,15i8),15i8));
  }
  #[test]
  fn it_cascades_accumulators_next_values() {
    let test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&0i8),((0i8,0i8),0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,15i8),15i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&7i8),((10i8,15i8),15i8));
  }
  #[test]
  fn it_cascades_accumulators_steps() {
    let mut test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.step_unwrap(&3i8),6i8);
    assert_eq!(test.state,(4i8,6i8));
    assert_ne!(test.step_unwrap(&3i8),6i8);
    assert_ne!(test.state,(4i8,6i8));
  }
  #[test]
  fn it_cascades_average2_next_values() {
    // Cascade needs to be Trait `StateMachine` compliant, for Average2
    // the OutputType in hardcoded as f64, thus it can only be f64
    let test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(0f64,0f64),&0f64),((0f64,0f64),0f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3f64,5f64),&7f64),((7f64,5f64),5f64));
  }
  #[test]
  fn it_cascades_average2_steps() {
    let mut test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.step_unwrap(&3f64),2f64);
    assert_eq!(test.state,(3f64,2f64));
    assert_eq!(test.step_unwrap(&2f64),2.25f64);
    assert_ne!(test.state,(3f64,2f64));
  }
  #[test]
  fn it_cascades_delay_to_increment() {
    let mut test: Cascade<Delay<i64>,Increment<i64>> = Cascade::new((100i64,1i64));
    assert_eq!(test.step_unwrap(&3i64),101i64);
    assert_eq!(test.state,(3i64,1i64));
    assert_eq!(test.step_unwrap(&2i64),4i64);
    assert_eq!(test.state,(2i64,1i64));
  }
  #[test]
  fn it_cascades_increment_to_delay() {
    let mut test: Cascade<Increment<i64>,Delay<i64>> = Cascade::new((1i64,100i64));
    assert_eq!(test.step_unwrap(&3i64),100i64);
    assert_eq!(test.state,(1i64,4i64));
    assert_eq!(test.step_unwrap(&2i64),4i64);
    assert_eq!(test.state,(1i64,3i64));
  }
  #[test]
  fn it_cascades_increment_to_delay_next_values_none() {
    let test: Cascade<Increment<i64>,Delay<i64>> = Cascade::new((2i64,3i64));
    assert_eq!(test.get_next_values(&(2i64,3i64),None),Ok(((2i64,3i64),Some(3i64))));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,3i64),&3i64),((2i64,5i64),3i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,5i64),&3i64),((2i64,5i64),5i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,5i64),&5i64),((2i64,7i64),5i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,7i64),&5i64),((2i64,7i64),7i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,7i64),&7i64),((2i64,9i64),7i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,9i64),&7i64),((2i64,9i64),9i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,9i64),&9i64),((2i64,11i64),9i64));
    assert_eq!(test.get_next_values_wrap_unwrap(&(2i64,11i64),&9i64),((2i64,11i64),11i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test: Cascade<Increment<i64>,Delay<i64>> = Cascade::new((2i64,3i64));
    assert_eq!(test.is_composite(),true);
  }
  #[test]
  fn it_forks_two_delays_plus_wire() {
    // Exercise 4.7 without Feedback
    let mut test:
      Cascade<
        Fork<       //Init|None  |1  |1  |1  |2  |3  |4  |6  |
          Wire<i64>,   //0|None>0|1>1|1>1|1>1|2>2|3>3|4>4|6>6|
          Cascade<     // |      |        
            Delay<i64>,//0|None>0|1>0|1>1|1>1|2>1|3>2|4>3|6>4|
            Delay<i64> //1|   0>1|0>0|1>0|1>1|1>1|2>1|3>2|4>3|
          >
        >,
        Adder<i64>     //0|     1|  1|  1|  2|  3|  4|  6|  9|
      > = StateMachine::new(((0i64,(0i64, 1i64)),0i64));

    assert_eq!(test.step(None,       true,0),Ok(Some(1i64)));
    assert_eq!(test.step(Some(&1i64),true,0),Ok(Some(1i64)));
    assert_eq!(test.step(Some(&1i64),true,0),Ok(Some(1i64)));
    assert_eq!(test.step(Some(&1i64),true,0),Ok(Some(2i64)));
    assert_eq!(test.step(Some(&2i64),true,0),Ok(Some(3i64)));
    assert_eq!(test.step(Some(&3i64),true,0),Ok(Some(4i64)));
    assert_eq!(test.step(Some(&4i64),true,0),Ok(Some(6i64)));
  }
}
