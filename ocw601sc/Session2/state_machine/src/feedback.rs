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
  /// `StateType`(S) = numbers
  type StateType = <SM>::StateType;
  /// `InputType`(I) = numbers
  type InputType = <SM>::InputType;
  /// `OutputType`(O) = numbers
  type OutputType = <SM>::OutputType;
  /// `initial_value`(_s0_) is usually 0;
  fn new(initial_value: Self::StateType) -> Self {
    Feedback {
      sm: <SM>::new(initial_value),
      state: initial_value,
    }
  }
  fn start(&mut self) {}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> 
  {
    let sm_next_value = self.sm.get_next_state(&state,inp)?;
    Ok(sm_next_value)
  }
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
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{{ {} {} }} Feedback {{ {} In/{} }}",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(None),
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
    "Feedback".to_string()
  }
  fn is_composite(&self) -> bool {
    true
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
  fn get_state(&self) -> Self::StateType{
    self.state
  }
}
// XXX: Add these functions on utils.rs.
//impl<SM> Feedback<SM>
//where SM: super::StateMachine,
//      SM: super::StateMachine<InputType=<SM as super::StateMachine>::OutputType>,
//      SM: super::StateMachine<StateType=<SM as super::StateMachine>::InputType>,
//      <SM>::StateType: Clone + Copy,
//{
//  pub fn makeCounter(init: i64, step: i64)
//  where SM: super::StateMachine
//  {
//    unimplemented!("Wait up!");
//    //let mut feedback: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((step,init));
//    //return sm.Feedback(sm.Cascade(Increment(step), sm.Delay(init)))
//  }
//}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use delay::Delay;
  use increment::Increment;
  use cascade::Cascade;
  #[test]
  fn it_feedbacks_cascades_increment_to_delay_next_val() {
    let test: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((2i64,3i64));
    assert_eq!(test.get_next_values(&(2i64,3i64),None),Ok(((2i64,5i64),Some(3i64))));
    assert_eq!(test.get_next_values(&(2i64,5i64),None),Ok(((2i64,7i64),Some(5i64))));
    assert_eq!(test.get_next_values(&(2i64,7i64),None),Ok(((2i64,9i64),Some(7i64))));
  }
  #[test]
  fn it_checks_is_composite() {
    let test: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((2i64,3i64));
    assert_eq!(test.is_composite(),true);
  }
}
