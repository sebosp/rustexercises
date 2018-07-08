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
  /// `StateType`(S) = Matches constituent machine's StateType
  type StateType = <SM>::StateType;
  /// `InputType`(I) = A tuple of its constituent machine's (Input,Input), this
  /// is technically equivalent to (Input,Output) since that's the point of the
  /// exercise, both InputType and OutputType must match.
  type InputType = (<SM>::InputType,Option<<SM>::OutputType>);
  /// `OutputType`(O) = numbers
  type OutputType = <SM>::OutputType;
  /// `initial_value`(_s0_) is usually 0;
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>
    where SM: super::StateMachine,
          SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
  {
    let sm_next_value = self.sm.get_next_values(&state,Some((inp,None)))?; // XXX: How do we pass the tuple (inp,None) to the internal StateMachine?
    let sm_feedback = self.sm.get_next_values(&state,Some((inp,sm_next_value.1.as_ref())))?;
    Ok((sm_feedback.0,sm_feedback.1))
  }
}
