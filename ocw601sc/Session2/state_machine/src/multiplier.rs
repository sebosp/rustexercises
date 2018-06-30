//! # Multiplier
//! A very simple machine who's input is two numbers, it returns the multiplication.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
// There is no reason for a struct to exist in this StateMachine.
pub struct Multiplier<T>
where T: Num + Clone + Copy + Display,
{
  pub state: T,
}
impl<T> super::StateMachine for Multiplier<T>
where T: Num + Clone + Copy + Display,
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = (Option<T>,Option<T>);
  /// `OutputType`(O) = numbers
  type OutputType = T;
  fn new(initial_value: Self::StateType) -> Self {
    Multiplier {
      state: initial_value
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    let inp = *inp;
    match inp.0 {
      None        =>
        match inp.1 {
          None        => Ok(T::one()), // Multiplicative Identity.
          Some(inp_1) => Ok(inp_1),
        },
      Some(inp_0) =>
        match inp.1 {
          None        => Ok(inp_0),
          Some(inp_1) => Ok(inp_0 * inp_1),
        },
    }
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,Some(*state))),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        // XXX: Check for infinity.
        Ok((next_state,Some(next_state)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} -> ({})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_input(inp),
             self.verbose_output(outp.1.as_ref()))
    }
    Ok(outp.1)
  }
  fn verbose_state(&self, _: &Self::StateType) -> String {
    format!("No State")
  }
  fn state_machine_name(&self) -> String {
    "Multiplier".to_string()
  }
  fn is_composite(&self) -> bool {
    true
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  =>
        match inp.0 {
          None        => 
            match inp.1 {
              None        => format!("In: (None,None)"),
              Some(inp_1) => format!("In: (None,{})",inp_1),
            }
          Some(inp_0) => 
            match inp.1 {
              None        => format!("In: ({},None)",inp_0),
              Some(inp_1) => format!("In: ({},{})",inp_0,inp_1),
            }
        }
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
  use accumulator::Accumulator;
  use delay::Delay;
  use fork::Fork;
  use cascade::Cascade;
  #[test]
  fn it_gets_next_values_input_some_none() {
    let test = Multiplier::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),None)),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(10i8),None)),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_input_none_some() {
    let test = Multiplier::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(10i8))),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_some() {
    let test = Multiplier::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(5i8),Some(7i8))),(35i8,35i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(1i8),Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(1i8))),(0i8,0i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Multiplier::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
    assert_eq!(test.get_next_values(&1i8,None),Ok((1i8,Some(1i8))));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Multiplier::new(0);
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(0i8))),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(1i8))),Ok(0i8));
    assert_eq!(test.get_next_state(&5i8,&(Some(3i8),Some(7i8))),Ok(21i8));
  }
  #[ignore]
  #[test]
  fn it_checks_is_composite() {
    let test = Multiplier::new(0);
    assert_eq!(test.is_composite(),true);
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_cascade() {
    let test: Cascade<Fork<Accumulator<i8>,Accumulator<i8>>,Multiplier<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&5i8),Ok(((7i8,8i8),56i8)));
  }
  #[ignore]
  #[test]
  fn it_gets_next_state_adder_from_forked_delays() {
    let test: Cascade<Fork<Delay<i8>,Delay<i8>>,Multiplier<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&7i8),Ok(((7i8,7i8),6i8)));
  }
}
