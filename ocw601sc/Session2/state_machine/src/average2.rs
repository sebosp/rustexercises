//! # Average2
//! A machine whose output is the average of the current input and the
//! previous input. It stores the new average as new state when it steps. 
//! State and Input should be i32 to allow for f64 conversion without
//! dropping of precision.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
pub struct Average2<T>
where T: Num + Display + Clone + Copy + FromPrimitive + ToPrimitive,
{
  pub state: T,
}
impl<T> super::StateMachine for Average2<T>
where T: Num + Display + Clone + Copy + FromPrimitive + ToPrimitive,
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = T;
  /// `OutputType`(O) = numbers
  type OutputType = f64;
  fn new(initial_value: Self::StateType) -> Self {
    Average2 {
      state: initial_value,
    }
  }
  fn start(&mut self){
    self.state = T::zero()
  }
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(*inp)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        let cast_avg_to_f64 = ToPrimitive::to_f64(&(*state + next_state));
        match cast_avg_to_f64 {
          Some(t) => if t.is_finite() {
            Ok((next_state,Some(t/2.0f64)))
          } else {
            Err("Adding numbers reached infinity".to_string())
          },
          None => Err("Unable to cast average to f64".to_string()),
        }
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: i8) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(self.state),
             self.verbose_input(inp),
             self.verbose_state(outp.0),
             self.verbose_output(outp.1))
    }
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}", state)
  }
  fn state_machine_name(&self) -> String {
    "Average2".to_string()
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  => format!("In: {}", inp),
    }
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: None"),
      Some(outp) => format!("Out: {}", outp),
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use std::f64;
  use std::i64;
  #[test]
  fn it_gets_next_values_i8() {
    let test = Average2::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&0i8),(0i8,0f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&1i8),(1i8,0.5f64));
  }
  #[test]
  fn it_steps_i8() {
    let mut test = Average2::new(0);
    assert_eq!(test.step_unwrap(&1i8),0.5f64);
    assert_eq!(test.step_unwrap(&1i8),1f64);
    assert_eq!(test.state,1i8);
  }
  #[test]
  #[should_panic(expected = "attempt to add with overflow")]
  fn it_gets_next_value_beyond_maxi64() {
    // XXX: overflow should be handled and return Err.
    let _test = Average2::new(0i64).get_next_values_wrap_unwrap(&(i64::MAX - 1i64),&(i64::MAX - 1i64));
  }
  #[test]
  fn it_gets_next_values_f64() {
    let test = Average2::new(0f64);
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&0f64),(0f64,0f64));
    assert_eq!(test.get_next_values_wrap_unwrap(&0f64,&1f64),(1f64,0.5f64));
  }
  #[test]
  fn it_gets_next_values_infinity() {
    let test = Average2::new(0f64);
    assert_eq!(test.get_next_values(&f64::MAX,Some(&f64::MAX)),Err("Adding numbers reached infinity".to_string()));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Average2::new(0);
    assert_eq!(test.get_next_state(&0i8,&0i8),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&1i8),Ok(1i8));
    assert_eq!(test.get_next_state(&0i8,&2i8),Ok(2i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Average2::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
