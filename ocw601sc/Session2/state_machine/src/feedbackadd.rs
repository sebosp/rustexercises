//! # FeedbackAdd
use std::fmt::Display;
extern crate num_traits;
use num_traits::*;
pub struct FeedbackAdd<SM,T>
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=T>,
        SM: super::StateMachine<OutputType=T>,
        <SM>::StateType: Clone + Copy,
        T: Num + Display + Clone + Copy + PartialEq,
{
  pub sm: SM,
  pub state: <SM>::StateType,
}
impl<SM,T> super::StateMachine for FeedbackAdd<SM,T>
  where SM: super::StateMachine,
        SM: super::StateMachine<InputType=T>,
        SM: super::StateMachine<OutputType=T>,
        <SM>::StateType: Clone + Copy,
        T: Num + Display + Clone + Copy + PartialEq,
{
  /// `StateType`(S) = Something inside constituent SM
  type StateType = <SM>::StateType;
  /// `InputType`(I) = Something inside constituent SM
  type InputType = T;
  /// `OutputType`(O) = Something inside constituent SM
  type OutputType = T;
  fn new(initial_value: Self::StateType) -> Self {
    FeedbackAdd {
      sm: <SM>::new(initial_value),
      state: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
     let sm_next_state = self.sm.get_next_state(&state,inp)?;
     Ok(sm_next_state)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None    => Err("The input of a FeedbackAdd StateMachine must not be None".to_string()),
      Some(inp1) => {
        let sm_next_value = self.sm.get_next_values(state,None)?;
        match sm_next_value.1 {
          None    => Err("The output of the Constituent Machine Feedback must not be None".to_string()),
          Some(inp2) => Ok((sm_next_value.0,Some(*inp1 + inp2)))
        }
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    match inp {
      None    => Err("The input of a FeedbackAdd StateMachine must not be None".to_string()),
      Some(val) => {
        let outp:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,None)?;
        if verbose {
          println!("{}{}::{{ {} {} }} FeedbackAdd {{ {} In/{} }}",
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
        match feedback.1 {
          Some(feedback_val) => Ok(Some(feedback_val + *val)),
          None => Ok(Some(T::zero())),
        }
      }
    }
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}",self.sm.verbose_state(state))
  }
  fn state_machine_name(&self) -> String {
    "FeedbackAdd".to_string()
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
/*#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  use accumulator::Accumulator;
  use delay::Delay;
  use fork::Fork;
  use cascade::Cascade;
  #[test]
  fn it_gets_next_values_input_some_none() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),None)),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(10i8),None)),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_input_none_some() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(10i8))),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_some() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(5i8),Some(7i8))),(12i8,12i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(1i8),Some(0i8))),(1i8,1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(1i8))),(1i8,1i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
    assert_eq!(test.get_next_values(&1i8,None),Ok((1i8,Some(1i8))));
  }
  #[test]
  fn it_gets_next_state() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(0i8))),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(1i8))),Ok(1i8));
    assert_eq!(test.get_next_state(&5i8,&(Some(3i8),Some(7i8))),Ok(10i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = FeedbackAdd::new(0);
    assert_eq!(test.is_composite(),false);
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_cascade() {
    let test: Cascade<Fork<Accumulator<i8>,Accumulator<i8>>,FeedbackAdd<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&5i8),Ok(((7i8,8i8),15i8)));
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_delays() {
    let test: Cascade<Fork<Delay<i8>,Delay<i8>>,FeedbackAdd<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&7i8),Ok(((7i8,7i8),5i8)));
  }
}*/
