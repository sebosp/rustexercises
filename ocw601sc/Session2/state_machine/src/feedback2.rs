//! # Feedback2 Composite StateMachine
//! It takes StateMachine with two Inputs and one Output.
//! It connects the Output of the internal machine to the second input of the
//! internal statemachine using the feedback loop.
//! Feedback2's InputType could be tuple (T,Option<Y>), Y must be OutputType
//! pseudo-code example:
//! 1. let (_,Y) = get_next_values(state, input, None);
//! 2. let (A,Z) = get_next_values(state, input, Some(Y)).
//! Y is both the second item in the tuple and the OutputType:
//!
//!  (InputType)                            (OutputType)
//!1. (i,None)            ---------------
//!               -----> | Internal      |---+--------------> O
//!                      | DualInput     |   |
//!2. (i,Some(Y))    .-> | StateMachine  |   | Initial O is fed back, call it Y
//!                  |    ---------------    |
//!                  |                       |
//!                  '-----------------------'
//!
//! When using the Feedback2, the internal StateMachine must be prepared to
//! receive None as one of the inputs and act accordingly.
//!
//! Is it possible to have something like this next snippet?
//! where SM: super::StateMachine<InputType.1=<SM as super::StateMachine>::OutputType>>
use std::fmt::Display;
pub struct Feedback2<SM,T>
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
        <SM>::StateType: Clone + Copy,
        <SM>::OutputType: Display,
{
  pub sm: SM,
  pub inp2: Option<T>,
  pub state: <SM>::StateType,
}
impl<SM,T> super::StateMachine for Feedback2<SM,T> 
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
        <SM>::StateType: Clone + Copy,
        <SM>::OutputType: Display,
{
  /// `StateType`(S) = numbers
  type StateType = <SM>::StateType;
  /// `InputType`(I) = numbers
  type InputType = (<SM>::InputType,Option<<SM>::OutputType>);
  /// `OutputType`(O) = numbers
  type OutputType = <SM>::OutputType;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Feedback2 {
      sm: <SM>::new(initial_value),
      inp2: None,
      state: initial_value,
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> 
  {
    Ok(*state) // XXX: Do nothing for now.
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>
    where SM: super::StateMachine,
          SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
  {
    let sm_next_value = self.sm.get_next_values(&state,Some((inp,None)))?; // XXX: How do we pass the tuple (inp,None) to the internal StateMachine?
    let sm_feedback = self.sm.get_next_values(&state,Some((inp,sm_next_value.1.as_ref())))?;
    Ok((sm_feedback.0,sm_feedback.1))
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{{ {} {} }} Feedback2 {{ {} In/{} }}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref())
             );
    }
    let feedback:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,outp.1.as_ref())?;
    let _ = self.sm.step(outp.1.as_ref(),verbose,depth+1)?;
    if verbose {
      println!("{}{}::{} {}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&feedback.0),
             self.verbose_output(feedback.1.as_ref())
             );
    }
    self.state = feedback.0;
    Ok(feedback.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}",self.sm.verbose_state(state))
  }
  fn state_machine_name(&self) -> String {
    "Feedback2".to_string()
  }
  fn is_composite(&self) -> bool {
    true
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    self.sm.verbose_input(inp)
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: None"),
      Some(outp) => format!("Out: {}", outp),
    }
  }
}
//impl<SM> Feedback2<SM>
//where SM: super::StateMachine,
//      SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
//      SM: super::StateMachine<StateType=<SM as super::StateMachine>::InputType>,
//      <SM>::StateType: Clone + Copy,
//{
//  pub fn makeCounter(init: i64, step: i64)
//  where SM: super::StateMachine
//  {
//    unimplemented!("Wait up!");
//    //let mut feedback: Feedback2<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((step,init));
//    //return sm.Feedback2(sm.Cascade(Increment(step), sm.Delay(init)))
//  }
//}
/*
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use delay::Delay;
  use fork::Fork;
  use increment::Increment;
  use cascade::Cascade;
  #[ignore]
  #[test]
  fn it_feedbacks_cascades_increment_to_delay_next_val() {
    let test: Feedback2<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((2i64,3i64));
    assert_eq!(test.get_next_values(&(2i64,3i64),None),Ok(((2i64,5i64),Some(3i64))));
    assert_eq!(test.get_next_values(&(2i64,5i64),None),Ok(((2i64,7i64),Some(5i64))));
    assert_eq!(test.get_next_values(&(2i64,7i64),None),Ok(((2i64,9i64),Some(7i64))));
  }
  #[test]
  fn it_checks_is_composite() {
    let test:
      Feedback2<
        Cascade<
          Fork<
            Increment<i64>,
            Delay<i64>
          >,
        Multiplier<i64,i64>
        >
      > = StateMachine::new(((1i64,2i64),(3i64,4i64)));
    assert_eq!(test.is_composite(),true);
  }
}*/
