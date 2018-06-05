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
    let cast_avg_to_f64 = ToPrimitive::to_f64(&(state + next_state));
    match cast_avg_to_f64 {
      Some(t) => if t.is_finite() {
        Ok((next_state,t/2.0f64))
      } else {
        Err("Adding numbers reached infinity".to_string())
      },
      None => Err("Unable to cast average to f64".to_string()),
    }
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
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
    assert_eq!(test.get_next_values(0i8,0i8),Ok((0i8,0f64)));
    assert_eq!(test.get_next_values(0i8,1i8),Ok((1i8,0.5f64)));
  }
  #[test]
  fn it_steps_i8() {
    let mut test = Average2::new(0);
    assert_eq!(test.step(&1i8),Ok(0.5f64));
    assert_eq!(test.step(&1i8),Ok(1f64));
    assert_eq!(test.state,1i8);
  }
  #[test]
  #[should_panic(expected = "attempt to add with overflow")]
  fn it_gets_next_value_beyond_maxi64() {
    // XXX: overflow should be handled and return Err.
    let _test = Average2::new(0i64).get_next_values(i64::MAX - 1i64,i64::MAX - 1i64);
  }
  #[test]
  fn it_gets_next_values_f64() {
    let test = Average2::new(0f64);
    assert_eq!(test.get_next_values(0f64,0f64),Ok((0f64,0f64)));
    assert_eq!(test.get_next_values(0f64,1f64),Ok((1f64,0.5f64)));
  }
  #[test]
  fn it_gets_next_values_infinity() {
    let test = Average2::new(0f64);
    assert_eq!(test.get_next_values(f64::MAX,f64::MAX),Err("Adding numbers reached infinity".to_string()));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Average2::new(0);
    assert_eq!(test.get_next_state(0i8,0i8),Ok(0i8));
    assert_eq!(test.get_next_state(0i8,1i8),Ok(1i8));
    assert_eq!(test.get_next_state(0i8,2i8),Ok(2i8));
  }
}
