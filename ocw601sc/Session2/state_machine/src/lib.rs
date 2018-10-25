//! # StateMachine crate
//!
//! `StateMachine` is a collection of StatesMachines as defined by MIT OWC601SC
//! State machines are a method of modeling systems whose output depends on the
//! entire history of their inputs, and not just on the most recent input.
//! Compared to purely functional systems, in which the output is purely
//! determined by the input, state machines have a performance that is 
//! determined by its history.
//! [Source](https://ocw.mit.edu/courses/electrical-engineering-and-computer-science/6-01sc-introduction-to-electrical-engineering-and-computer-science-i-spring-2011/unit-1-software-engineering/state-machines/MIT6_01SCS11_chap04.pdf)
extern crate num_traits;
// Simple machines:
pub mod accumulator;
pub mod gain;
pub mod abc;
pub mod updown;
pub mod delay;
pub mod average2;
pub mod sumlast3;
pub mod selector;
pub mod simple_parking_gate;
pub mod increment;
pub mod negation;
pub mod wire;
// Composite machines:
pub mod adder;
pub mod multiplier;
pub mod cascade;
pub mod parallel;
pub mod fork;
pub mod feedback;
pub mod feedback2;
pub trait StateMachine {
  type StateType;
  type InputType;
  type OutputType;
  fn new(initial_value: Self::StateType) -> Self;
  /// `start` creates an attribute of the instance, called state, and assign
  /// to it the value of the startState attribute. It is crucial that we have
  /// both of these attributes: if we were to just modify startState, then if
  /// we wanted to run this machine again, we would have permanently forgotten
  /// what the starting state should be. 
  /// Not all types of StateMachine use start and not all of them have a state
  fn start(&mut self);
  /// `get_current_state` returns the current state of the State Machine.
  fn get_current_state(&self) -> Self::StateType;
  /// `get_next_state` given an input and a state will return the next state.
  /// the returned value will be treated both as the output and the next state
  /// of the machine, `get_next_values` function uses it to compute both values
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String>;
  /// `get_next_values` defines both the next-state function and the output
  /// function, by taking the current state and input as arguments and
  /// returning a tuple containing both the next state and the output.
  /// It is crucial that `get_next_values` be a pure function; that is,
  /// that it not change the state of the object (by assigning to any
  /// attributes of self). It must simply compute the necessary values and
  /// return them. We do not promise anything about how many times this method
  /// will be called and in what circumstances.
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>;
  /// `step` changes the current state and goes through the mutable process
  /// of the StateMachine progression. It expects a verbose flag and a depth flag.
  /// The depth flag helps debugging complex big State Machines when they are
  /// composite machines.
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String>;
  /// Helper function that wraps the input in a Some() and unwraps the Result
  /// Panics on None
  fn get_next_values_wrap_unwrap(&self, state: &Self::StateType, inp: &Self::InputType) -> (Self::StateType,Self::OutputType) {
    let res = self.get_next_values(state,Some(inp));
    match res {
      Err(x) => panic!("get_next_values_wrap_unwrap got Err({})",x),
      Ok(opt) => {
        match opt.1 {
          None => panic!("get_next_values_wrap_unwrap got None"),
          Some(val) => (opt.0,val)
        }
      }
    }
  }
  /// Helper function that unwraps the Result. Panics on Err/None
  fn step_unwrap(&mut self, inp: &Self::InputType) -> Self::OutputType {
    let res = self.step(Some(inp), false, 0usize);
    match res {
      Err(x) => panic!("step_unwrap got Err({})",x),
      Ok(opt) => {
        match opt {
          None => panic!("step_unwrap got None"),
          Some(val) => val
        }
      }
    }
  }
  /// A transducer is a process that takes as input a sequence of values which
  /// serve as inputs to the state machine, and returns as ouput the set of
  /// outputs of the machine for each input
  fn transduce(&mut self, inp: Vec<Option<Self::InputType>>, verbose: bool, _: bool) -> Vec<Result<Option<Self::OutputType>, String>> {
    let depth = 0usize; // The depth for verbose printing show indent
    let mut res: Vec<Result<Option<Self::OutputType>, String>> = Vec::new();
    for cur_inp in inp {
      match self.step(cur_inp.as_ref(), verbose, depth) {
        Ok(cur_out) => {
          res.push(Ok(cur_out));
        },
        Err(e) => {
          res.push(Err(e));
        }
      };
    }
    res
  }
  /// A transducer_wrap_unwrap wraps the Input into an Option and unwraps the Output out of Option
  /// this is an unsafe version, will panic on step result items being None
  fn transduce_wrap_unwrap(&mut self, inp: Vec<Self::InputType>, verbose: bool, _: bool) -> Vec<Result<Self::OutputType, String>> {
    let mut unwrapped_res: Vec<Result<Self::OutputType, String>> = Vec::new();
    let mut wrapped_inp: Vec<Option<Self::InputType>> = Vec::new();
    for cur_inp in inp {
      wrapped_inp.push(Some(cur_inp));
    }
    let res_transduce = self.transduce(wrapped_inp, verbose, false);
    for cur_res in res_transduce {
      match cur_res {
        Ok(good_res) => {
          match good_res {
            None    => panic!("transduce_wrap_unwrap does not support None responses"),
            Some(x) => unwrapped_res.push(Ok(x))
          };
        },
        Err(e) => unwrapped_res.push(Err(e))
      };
    }
    unwrapped_res
  }
  fn state_machine_name(&self) -> String {
    "UNSET".to_string()
  }
  /// Ideally verbose input and output should have a default implementation
  /// here were it simply works for simple types that implement Display.
  fn verbose_state(&self, state: &Self::StateType) -> String;
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String;
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String;
  /// StateMachines register themselves as Composites on Constituent
  /// They are non-composite machines by default.
  /// When a StateMachine is Composite, its output/input is allowed to be None.
  fn is_composite(&self) -> bool {
    false
  }
}
/// Some functions expect two Inputs, for example when it joins the output
/// of two State Machines:
pub trait DualInput {
  type T1;
  type T2;
}
impl<T1,T2> DualInput for DualValues<T1,T2> {
  type T1 = T1;
  type T2 = T2;
}
#[derive(Debug)]
pub struct DualValues<T1,T2> {
  pub val1: Option<T1>,
  pub val2: Option<T2>,
}
impl<T1,T2> PartialEq for DualValues<T1,T2>
where T1: PartialEq,
      T2: PartialEq,
{
  fn eq(&self, rhs: &DualValues<T1,T2>) -> bool
  where T1: PartialEq,
      T2: PartialEq,
  {
    let val1cmp = match self.val1 {
      Some(ref lval1) => match rhs.val1 {
        None       => false,
        Some(ref rval1) => {
          lval1 == rval1
        },
      },
      None       => match rhs.val1 {
        None    => true,
        Some(_) => false,
      }
    };
    let val2cmp = match self.val2 {
      Some(ref lval2) => match rhs.val2 {
        None       => false,
        Some(ref rval2) => {
          lval2 == rval2
        },
      },
      None       => match rhs.val2 {
        None    => true,
        Some(_) => false,
      }
    };
    (val1cmp == true && val1cmp == val2cmp)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_compares_eq_dualvalues() {
    let lhs = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    let rhs = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    assert_eq!(lhs,rhs);
    let lhs:DualValues<u8,u8> = DualValues{ val1: None, val2: None};
    let rhs:DualValues<u8,u8> = DualValues{ val1: None, val2: None};
    assert_eq!(lhs,rhs);
    let lhs:DualValues<u8,u8> = DualValues{ val1: Some(3), val2: None};
    let rhs :DualValues<u8,u8>= DualValues{ val1: Some(3), val2: None};
    assert_eq!(lhs,rhs);
    let lhs:DualValues<u8,String> = DualValues{ val1: None, val2: Some("a".to_string())};
    let rhs:DualValues<u8,String> = DualValues{ val1: None, val2: Some("a".to_string())};
    assert_eq!(lhs,rhs);
  }
  #[test]
  fn it_compares_ne_dualvalues() {
    let lhs = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    let rhs = DualValues{ val1: Some(7), val2: Some("a".to_string())};
    assert_ne!(lhs,rhs);
    let lhs = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    let rhs = DualValues{ val1: Some(3), val2: Some("b".to_string())};
    assert_ne!(lhs,rhs);
    let lhs:DualValues<u8,String> = DualValues{ val1: None,    val2: Some("a".to_string())};
    let rhs:DualValues<u8,String> = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    assert_ne!(lhs,rhs);
    let lhs:DualValues<u8,String> = DualValues{ val1: Some(3), val2: None};
    let rhs = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    assert_ne!(lhs,rhs);
    let lhs:DualValues<u8,String> = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    let rhs:DualValues<u8,String> = DualValues{ val1: None,    val2: Some("a".to_string())};
    assert_ne!(lhs,rhs);
    let lhs:DualValues<u8,String> = DualValues{ val1: Some(3), val2: Some("a".to_string())};
    let rhs:DualValues<u8,String> = DualValues{ val1: Some(3), val2: None};
    assert_ne!(lhs,rhs);
  }
}
