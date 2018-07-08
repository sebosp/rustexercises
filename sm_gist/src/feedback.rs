//! # Feedback StateMachine
//! The initial input of this StateMachine is None, 
//! Inspecting the current state of the constituent machine
//! yields the Input for itself.
use std::fmt::Display;
pub struct Feedback<SM>
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
        <SM>::StateType: Clone + Copy,
        <SM>::OutputType: Display,
{
  pub sm: SM,
  pub state: <SM>::StateType,
}
impl<SM> super::StateMachine for Feedback<SM> 
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
        <SM>::StateType: Clone + Copy,
        <SM>::OutputType: Display,
{
  /// `StateType`(S) = Constituent Machine's StateType
  type StateType = <SM>::StateType;
  /// `InputType`(I) = Constituent Machine's InputType
  type InputType = <SM>::InputType;
  /// `OutputType`(O) = Constituent Machine's OutputType
  type OutputType = <SM>::OutputType;
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String>
    where SM: super::StateMachine,
          SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
  {
    match inp {
      Some(_) => Err("The input of a Feedback StateMachine must be None".to_string()),
      None => {
        let sm_next_value = self.sm.get_next_values(&state,None)?;
        let sm_feedback = self.sm.get_next_values(&sm_next_value.0,sm_next_value.1.as_ref())?;
        match sm_feedback.1 {
          None    => Err("The output of the Constituent Machine 2nd run must not be None".to_string()),
          Some(sm_feedback_val) => Ok((sm_feedback.0,Some(sm_feedback_val)))
        }
      }
    }
  }
}
