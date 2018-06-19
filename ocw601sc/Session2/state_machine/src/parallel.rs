//! # Parallel Composite StateMachine
//! One input feeds two StateMachines, not to be confused with parallel
//! execution (yet), both StateMachine Input is the same type/value.
//! This could be thought of as a fork
pub struct Parallel<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: SM1,
  pub sm2: SM2,
  pub state: (<SM1>::StateType,<SM2>::StateType),
}
impl<SM1,SM2> super::StateMachine for Parallel<SM1,SM2> 
  where SM1: super::StateMachine,
        SM2: super::StateMachine,
        <SM1>::StateType: Clone + Copy,
        <SM2>::StateType: Clone + Copy,
        <SM1>::InputType: Clone + Copy,
        <SM2>::InputType: Clone + Copy,
{
  /// `StateType`(S) = numbers
  type StateType = (<SM1>::StateType,<SM2>::StateType);
  /// `InputType`(I) = numbers
  type InputType = (<SM1>::InputType,<SM2>::InputType);
  /// `OutputType`(O) = numbers
  type OutputType = (<SM1>::OutputType,<SM2>::OutputType);
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Parallel {
      sm1: <SM1>::new(initial_value.0),
      sm2: <SM2>::new(initial_value.1),
      state: initial_value,
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    let sm1_next_state = self.sm1.get_next_state(&state.0,&inp.0)?;
    let sm2_next_state = self.sm2.get_next_state(&state.1,&inp.1)?;
    Ok((sm1_next_state,sm2_next_state))
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None      => Ok((*state,None)),
      Some(inp) => {
        let sm1_next_values = self.sm1.get_next_values(&state.0,Some(&inp.0))?;
        match sm1_next_values.1 {
          None               => Err("Parallel::FIXME:XXX:TODO".to_string()),
          Some(sm1_next_val) => {
            let sm2_next_values = self.sm2.get_next_values(&state.1,Some(&inp.1))?;
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
  fn step(&mut self, inp: Option<&Self::InputType>) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("State: (SM1:{}, SM2:{})",self.sm1.verbose_state(),self.sm2.verbose_state())
  }
  fn state_machine_name(&self) -> String {
    "Parallel".to_string()
  }
  fn verbose_step(&self, inp: Option<&Self::InputType>, outp: Option<&Self::OutputType>) -> String {
    format!("{}: {} {} (SM1:{}, SM2:{}) {}", self.state_machine_name(), self.verbose_input(inp),self.verbose_output(outp), self.sm1.verbose_state(),self.sm2.verbose_state(), self.verbose_state())
  }
  fn is_composite(&self) -> bool {
    true
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  => format!("In: (SM1: {},SM2: {})", self.sm1.verbose_input(Some(&inp.0)), self.sm2.verbose_input(Some(&inp.1))),
    }
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: None"),
      Some(outp) => format!("Out: ({},{})",self.sm1.verbose_output(Some(&outp.0)),self.sm2.verbose_output(Some(&outp.1)))
    }
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
    let test: Parallel<Accumulator<i8>,Accumulator<i8>> = Parallel::new((1i8,2i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&(0i8,0i8)),((0i8,0i8),(0i8,0i8)));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&(7i8,7i8)),((10i8,12i8),(10i8,12i8)));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&(7i8,7i8)),((10i8,12i8),(10i8,12i8)));
  }
  #[test]
  fn it_get_next_state_accumulators() {
    let test: Parallel<Accumulator<i8>,Accumulator<i8>> = Parallel::new((1i8,2i8));
    assert_eq!(test.get_next_state(&(0i8,0i8),&(0i8,0i8)),Ok((0i8,0i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&(7i8,7i8)),Ok((10i8,12i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&(7i8,7i8)),Ok((10i8,12i8)));
  }
  #[test]
  fn it_steps_accumulators() {
    let mut test: Parallel<Accumulator<i8>,Accumulator<i8>> = Parallel::new((1i8,2i8));
    assert_eq!(test.step_unwrap(&(3i8,3i8)),(4i8,5i8));
    assert_eq!(test.state,(4i8,5i8));
    assert_eq!(test.step_unwrap(&(5i8,5i8)),(9i8,10i8));
    assert_eq!(test.state,(9i8,10i8));
  }
  #[test]
  fn it_steps_increments() {
    let mut test: Parallel<Increment<i64>,Increment<i64>> = Parallel::new((100i64,1i64));
    assert_eq!(test.step_unwrap(&(3i64,3i64)),(103i64,4i64));
    assert_eq!(test.state,(100i64,1i64));
    assert_eq!(test.step_unwrap(&(2i64,2i64)),(102i64,3i64));
    assert_eq!(test.state,(100i64,1i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test: Parallel<Accumulator<i8>,Accumulator<i8>> = Parallel::new((1i8,2i8));
    assert_eq!(test.is_composite(),true);
  }
}
