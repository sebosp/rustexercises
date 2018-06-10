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
pub mod cascade;
pub mod parallel;
pub mod fork;
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
  fn get_next_values(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<(Self::StateType,Self::OutputType),String>;
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String>;
  fn verbose_state(&self) -> String;
  fn verbose_step(&self, inp: &Self::InputType, outp: &Self::OutputType) -> String;
  /// A transducer is a process that takes as input a sequence of values which
  /// serve as inputs to the state machine, and returns as ouput the set of
  /// outputs of the machine for each input
  fn transduce(&mut self, inp: Vec<Self::InputType>, verbose: bool, _: bool) -> Vec<Result<Self::OutputType, String>> {
    let mut res: Vec<Result<Self::OutputType, String>> = Vec::new();
    if verbose {
      self.verbose_state();
    }
    for cur_inp in inp {
      match self.step(&cur_inp) {
        Ok(cur_out) => {
          if verbose {
            self.verbose_step(&cur_inp,&cur_out);
          }
          res.push(Ok(cur_out));
        },
        Err(e) => {
          res.push(Err(e));
        }
      };
    }
    res
  }
}
