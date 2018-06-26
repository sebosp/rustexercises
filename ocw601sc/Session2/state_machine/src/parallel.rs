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
    let sm1_res = match inp {
      None      => self.sm1.get_next_values(&state.0,None)?,
      Some(val) => self.sm1.get_next_values(&state.0,Some(&val.0))?,
    };
    let sm2_res = match inp {
      None      => self.sm2.get_next_values(&state.1,None)?,
      Some(val) => self.sm2.get_next_values(&state.1,Some(&val.1))?,
    };
    match sm1_res.1 {
      None      => {
        match sm2_res.1 {
          None    => Ok(((sm1_res.0,sm2_res.0),None)),
          Some(_) => Err("Parallel:XXX:Got different Option types for Output, SM1 is None, SM2 is Some()".to_string()),
        }
      },
      Some(sm1_res_outp) => {
        match sm2_res.1 {
          None               => Err("Parallel:XXX:Got different Option types for Output, SM1 is Some(), SM2 is None".to_string()),
          Some(sm2_res_outp) => Ok(((sm1_res.0,sm2_res.0),Some((sm1_res_outp,sm2_res_outp)))),
        }
      }
    }
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
