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
    match sm1_next_value.1 {
      None               => Err("FIXME:XXX:TODO".to_string()),
      Some(sm1_next_val) => {
        let sm2_next_state = self.sm2.get_next_state(&state.1,&sm1_next_val)?;
        Ok((sm1_next_value.0,sm2_next_state))
      }
    }
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>
  where SM1: super::StateMachine<OutputType=<SM2>::InputType>,
  {
    let sm1_next_value = self.sm1.get_next_values(&state.0,inp)?;
    match sm1_next_value.1 {
      None               => {
        let sm2_next_value = self.sm2.get_next_values(&state.1,None)?;
        Ok(((sm1_next_value.0,sm2_next_value.0),sm2_next_value.1))
      }
      Some(sm1_next_val) => {
        let sm2_next_value = self.sm2.get_next_values(&state.1,Some(&sm1_next_val))?;
        Ok(((sm1_next_value.0,sm2_next_value.0),sm2_next_value.1))
      }
    }
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,Some(inp))?;
    match outp.1 {
      None           => Err("FIXME:XXX:TODO".to_string()),
      Some(next_val) => {
        self.state = outp.0;
        Ok(next_val)
      }
    }
  }
  fn verbose_state(&self) -> String {
    format!("Start state: (SM1:{}, SM2:{})",self.sm1.verbose_state(),self.sm2.verbose_state())
  }
  fn verbose_step(&self, _: &Self::InputType, _: &Self::OutputType) -> String {
    format!("Step: (SM1:{}, SM2:{})",self.sm1.verbose_state(),self.sm2.verbose_state())
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
  #[test]
  fn it_cascades_accumulators_next_values() {
    let test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.get_next_values(&(0i8,0i8),Some(&0i8)),Ok(((0i8,0i8),Some(0i8))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,15i8),Some(15i8))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,15i8),Some(15i8))));
  }
  #[test]
  fn it_cascades_accumulators_steps() {
    let mut test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.step(&3i8),Ok(6i8));
    assert_eq!(test.state,(4i8,6i8));
    assert_ne!(test.step(&3i8),Ok(6i8));
    assert_ne!(test.state,(4i8,6i8));
  }
  #[test]
  fn it_cascades_average2_next_values() {
    // Cascade needs to be Trait `StateMachine` compliant, for Average2
    // the OutputType in hardcoded as f64, thus it can only be f64
    let test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.get_next_values(&(0f64,0f64),Some(&0f64)),Ok(((0f64,0f64),Some(0f64))));
    assert_eq!(test.get_next_values(&(3f64,5f64),Some(&7f64)),Ok(((7f64,5f64),Some(5f64))));
  }
  #[test]
  fn it_cascades_average2_steps() {
    let mut test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.step(&3f64),Ok(2f64));
    assert_eq!(test.state,(3f64,2f64));
    assert_eq!(test.step(&2f64),Ok(2.25f64));
    assert_ne!(test.state,(3f64,2f64));
  }
  #[test]
  fn it_cascades_delay_to_increment() {
    let mut test: Cascade<Delay<i64>,Increment<i64>> = Cascade::new((100i64,1i64));
    assert_eq!(test.step(&3i64),Ok(101i64));
    assert_eq!(test.state,(3i64,1i64));
    assert_eq!(test.step(&2i64),Ok(4i64));
    assert_eq!(test.state,(2i64,1i64));
  }
  #[test]
  fn it_cascades_increment_to_delay() {
    let mut test: Cascade<Increment<i64>,Delay<i64>> = Cascade::new((1i64,100i64));
    assert_eq!(test.step(&3i64),Ok(100i64));
    assert_eq!(test.state,(1i64,4i64));
    assert_eq!(test.step(&2i64),Ok(4i64));
    assert_eq!(test.state,(1i64,3i64));
  }
  #[test]
  fn it_cascades_increment_to_delay_next_values_none() {
    let test: Cascade<Increment<i64>,Delay<i64>> = Cascade::new((2i64,3i64));
    assert_eq!(test.get_next_values(&(2i64,3i64),None),Ok(((2i64,3i64),Some(3i64))));
    assert_eq!(test.get_next_values(&(2i64,3i64),Some(&3i64)),Ok(((2i64,5i64),Some(3i64))));
    assert_eq!(test.get_next_values(&(2i64,5i64),Some(&3i64)),Ok(((2i64,5i64),Some(5i64))));
    assert_eq!(test.get_next_values(&(2i64,5i64),Some(&5i64)),Ok(((2i64,7i64),Some(5i64))));
    assert_eq!(test.get_next_values(&(2i64,7i64),Some(&5i64)),Ok(((2i64,7i64),Some(7i64))));
    assert_eq!(test.get_next_values(&(2i64,7i64),Some(&7i64)),Ok(((2i64,9i64),Some(7i64))));
    assert_eq!(test.get_next_values(&(2i64,9i64),Some(&7i64)),Ok(((2i64,9i64),Some(9i64))));
    assert_eq!(test.get_next_values(&(2i64,9i64),Some(&9i64)),Ok(((2i64,11i64),Some(9i64))));
    assert_eq!(test.get_next_values(&(2i64,11i64),Some(&9i64)),Ok(((2i64,11i64),Some(11i64))));
  }
}
