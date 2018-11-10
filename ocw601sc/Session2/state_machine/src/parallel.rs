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
  type OutputType = (Option<<SM1>::OutputType>,Option<<SM2>::OutputType>);
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
    let sm1_res = match inp {
      None      => self.sm1.get_next_values(&state.0,None)?,
      Some(val) => self.sm1.get_next_values(&state.0,Some(&val.0))?,
    };
    let sm2_res = match inp {
      None      => self.sm2.get_next_values(&state.1,None)?,
      Some(val) => self.sm2.get_next_values(&state.1,Some(&val.1))?,
    };
    Ok(((sm1_res.0,sm2_res.0),Some((sm1_res.1,sm2_res.1))))
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
    match inp {
      None      => {
        let _ = self.sm1.step(None,verbose,depth+1)?;
        let _ = self.sm2.step(None,verbose,depth+1)?;
      }
      Some(inp) => {
        let _ = self.sm1.step(Some(&inp.0),verbose,depth+1)?;
        let _ = self.sm2.step(Some(&inp.1),verbose,depth+1)?;
      }
    };
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
    format!("State: (SM1:{}, SM2:{})",self.sm1.verbose_state(&state.0),self.sm2.verbose_state(&state.1))
  }
  fn state_machine_name(&self) -> String {
    "Parallel".to_string()
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
      Some(outp) => format!("Out: ({},{})",self.sm1.verbose_output(outp.0.as_ref()),self.sm2.verbose_output(outp.1.as_ref()))
    }
  }
  fn get_state(&self) -> Self::StateType{
    self.state
  }
}
pub struct ParallelBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine
{
  pub sm1: Option<SM1>,
  pub sm2: Option<SM2>,
  pub state: (Option<<SM1>::StateType>,Option<<SM2>::StateType>),
}
impl<SM1,SM2> ParallelBuilder<SM1,SM2>
  where SM1: super::StateMachine,
        SM2: super::StateMachine,
        <SM1>::StateType: Clone + Copy,
        <SM2>::StateType: Clone + Copy,
        <SM1>::InputType: Clone + Copy,
        <SM2>::InputType: Clone + Copy,
{
  pub fn new() -> ParallelBuilder<SM1,SM2> {
    ParallelBuilder{
      sm1: None,
      sm2: None,
      state: (None,None),
    }
  }
  pub fn with_path1(mut self, input: SM1) -> ParallelBuilder<SM1,SM2> {
    self.state.0 = Some(input.get_state());
    self.sm1 = Some(input);
    self
  }
  pub fn with_path2(mut self, input: SM2) -> ParallelBuilder<SM1,SM2> {
    self.state.1 = Some(input.get_state());
    self.sm2 = Some(input);
    self
  }
  pub fn build(self) -> Result<Parallel<SM1,SM2>,String> {
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
      None => return Err("Missing 1st State Machine definition (with_path1)".to_string()),
    };
    let sm2 = match self.sm2 {
      Some(val) => val,
      None => return Err("Missing 2nd State Machine Definition (with_path2)".to_string()),
    };
    Ok(Parallel{
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
    let test = ParallelBuilder::new()
      .with_path1(Accumulator::new(1i8))
      .with_path2(Accumulator::new(2i8))
      .build().unwrap();
    assert_eq!(test.get_next_values_wrap_unwrap(&(0i8,0i8),&(0i8,0i8)),((0i8,0i8),(Some(0i8),Some(0i8))));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&(7i8,7i8)),((10i8,12i8),(Some(10i8),Some(12i8))));
    assert_eq!(test.get_next_values_wrap_unwrap(&(3i8,5i8),&(7i8,7i8)),((10i8,12i8),(Some(10i8),Some(12i8))));
  }
  #[test]
  fn it_get_next_state_accumulators() {
    let test = ParallelBuilder::new()
      .with_path1(Accumulator::new(1i8))
      .with_path2(Accumulator::new(2i8))
      .build().unwrap();
    assert_eq!(test.get_next_state(&(0i8,0i8),&(0i8,0i8)),Ok((0i8,0i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&(7i8,7i8)),Ok((10i8,12i8)));
    assert_eq!(test.get_next_state(&(3i8,5i8),&(7i8,7i8)),Ok((10i8,12i8)));
  }
  #[test]
  fn it_steps_accumulators() {
    let mut test = ParallelBuilder::new()
      .with_path1(Accumulator::new(1i8))
      .with_path2(Accumulator::new(2i8))
      .build().unwrap();
    assert_eq!(test.step_unwrap(&(3i8,3i8)),(Some(4i8),Some(5i8)));
    assert_eq!(test.state,(4i8,5i8));
    assert_eq!(test.step_unwrap(&(5i8,5i8)),(Some(9i8),Some(10i8)));
    assert_eq!(test.state,(9i8,10i8));
  }
  #[test]
  fn it_steps_increments() {
    let mut test = ParallelBuilder::new()
      .with_path1(Increment::new(100i64))
      .with_path2(Increment::new(1i64))
      .build().unwrap();
    assert_eq!(test.step_unwrap(&(3i64,3i64)),(Some(103i64),Some(4i64)));
    assert_eq!(test.state,(100i64,1i64));
    assert_eq!(test.step_unwrap(&(2i64,2i64)),(Some(102i64),Some(3i64)));
    assert_eq!(test.state,(100i64,1i64));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = ParallelBuilder::new()
      .with_path1(Accumulator::new(1i8))
      .with_path2(Accumulator::new(2i8))
      .build().unwrap();
    assert_eq!(test.is_composite(),true);
  }
}
