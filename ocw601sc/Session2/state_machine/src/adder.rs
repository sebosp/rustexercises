//! # Adder
//! A very simple machine who's input is two numbers, it returns the sum.
extern crate num_traits;
use num_traits::*;
use std::fmt::Display;
// There is no reason for a struct to exist in this StateMachine.
pub struct Adder<T>
where T: Num + Clone + Copy + Display,
{
  pub state: T,
}
impl<T> super::StateMachine for Adder<T>
where T: Num + Clone + Copy + Display,
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = super::DualValues<T,T>;
  /// `OutputType`(O) = numbers
  type OutputType = T;
  fn new(_: Self::StateType) -> Self {
    Adder {
      state: T::zero(),
    }
  }
  fn start(&mut self){
    self.state = T::zero()
  }
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    match inp.val1 {
      None        =>
        match inp.val2 {
          None        => Ok(T::zero()), // Additive Identity.
          Some(inp_2) => Ok(inp_2),
        },
      Some(inp_1) =>
        match inp.val2 {
          None        => Ok(inp_1),
          Some(inp_2) => Ok(inp_1 + inp_2),
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
    "Adder".to_string()
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  =>
        match inp.val1 {
          None        => 
            match inp.val2 {
              None        => format!("In: (None,None)"),
              Some(inp_2) => format!("In: (None,{})",inp_2),
            }
          Some(inp_1) => 
            match inp.val2 {
              None        => format!("In: ({},None)",inp_1),
              Some(inp_2) => format!("In: ({},{})",inp_1,inp_2),
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
  fn get_state(&self) -> Self::StateType{
    self.state
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
    let test = Adder::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(0i8), val2: None}),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(10i8),val2: None}),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_input_none_some() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: None, val2: Some(0i8)}),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: None, val2: Some(10i8)}),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_some() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(0i8), val2: Some(0i8)}),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(5i8), val2: Some(7i8)}),(12i8,12i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(1i8), val2: Some(0i8)}),(1i8,1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&DualValues{ val1: Some(0i8), val2: Some(1i8)}),(1i8,1i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
    assert_eq!(test.get_next_values(&1i8,None),Ok((1i8,Some(1i8))));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_state(&0i8,&DualValues{ val1: Some(0i8), val2: Some(0i8)}),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&DualValues{ val1: Some(0i8), val2: Some(1i8)}),Ok(1i8));
    assert_eq!(test.get_next_state(&5i8,&DualValues{ val1: Some(3i8), val2: Some(7i8)}),Ok(10i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Adder::new(0);
    assert_eq!(test.is_composite(),false);
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_cascade() {
    let test: Cascade<Fork<Accumulator<i8>,Accumulator<i8>>,Adder<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&5i8),Ok(((7i8,8i8),15i8)));
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_delays() {
    let test: Cascade<Fork<Delay<i8>,Delay<i8>>,Adder<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&7i8),Ok(((7i8,7i8),5i8)));
  }
}
