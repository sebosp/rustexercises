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
{
  /// `StateType`(S) = numbers
  type StateType = (<SM1>::StateType,<SM2>::StateType);
  /// `InputType`(I) = numbers
  type InputType = <SM1>::InputType;
  /// `OutputType`(O) = numbers
  type OutputType = (<SM1>::OutputType,<SM2>::OutputType);
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
    match inp {
      None      => Ok((*state,None)),
      Some(inp) => {
        let sm1_next_values = self.sm1.get_next_values(&state.0,Some(inp))?;
        match sm1_next_values.1 {
          None               => Err("FIXME:XXX:TODO".to_string()),
          Some(sm1_next_val) => {
            let sm2_next_values = self.sm2.get_next_values(&state.1,Some(inp))?;
            match sm2_next_values.1 {
              None               => Err("FIXME:XXX:TODO".to_string()),
              Some(sm2_next_val) => {
                Ok(((sm1_next_values.0,sm2_next_values.0),Some((sm1_next_val,sm2_next_val))))
              }
            }
          }
        }
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
  use increment::Increment;
  #[test]
  fn it_get_next_values_accumulators() {
    let test: Fork<Accumulator<i8>,Accumulator<i8>> = Fork::new((1i8,2i8));
    assert_eq!(test.get_next_values(&(0i8,0i8),Some(&0i8)),Ok(((0i8,0i8),Some((0i8,0i8)))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,12i8),Some((10i8,12i8)))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,12i8),Some((10i8,12i8)))));
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
    assert_eq!(test.step(&3i8),Ok((4i8,5i8)));
    assert_eq!(test.state,(4i8,5i8));
    assert_eq!(test.step(&5i8),Ok((9i8,10i8)));
    assert_eq!(test.state,(9i8,10i8));
  }
  #[test]
  fn it_steps_increments() {
    let mut test: Fork<Increment<i64>,Increment<i64>> = Fork::new((100i64,1i64));
    assert_eq!(test.step(&3i64),Ok((103i64,4i64)));
    assert_eq!(test.state,(100i64,1i64));
    assert_eq!(test.step(&2i64),Ok((102i64,3i64)));
    assert_eq!(test.state,(100i64,1i64));
  }
}
