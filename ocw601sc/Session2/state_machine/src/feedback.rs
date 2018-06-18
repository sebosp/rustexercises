//! # Cascade StateMachine
//! The input of a StateMachine becomes the output of the second StateMachine.
//use cascade::Cascade;
//use increment::Increment;
//use delay::Delay;
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
    println!("Feedback::get_next_values");
    match inp {
      Some(_) => Err("The input of a Feedback StateMachine must be None".to_string()),
      None => {
        println!("Feedback(None) state: {}",self.verbose_state());
        let sm_next_value = self.sm.get_next_values(&state,None)?;
        match sm_next_value.1 {
          None => {
            if self.sm.is_composite() {
              println!("Feedback(None) state: {}",self.verbose_state());
              let sm_feedback = self.sm.get_next_values(&state,None)?;
              match sm_feedback.1 {
                None    =>
                  if self.sm.is_composite() {
                    Ok((sm_feedback.0,None))
                  } else {
                    Err("The output of the Constituent Machine 2nd run must not be None".to_string())
                  },
                Some(sm_feedback_val) => Ok((sm_feedback.0,Some(sm_feedback_val)))
              }
            } else {
              Err("The output of the Constituent Machine 1st run must not be None".to_string())
            }
          },
          Some(sm_next_val) => {
            println!("Feedback(Some) state: {}",self.verbose_state());
            let sm_feedback = self.sm.get_next_values(&state,Some(&sm_next_val))?;
            match sm_feedback.1 {
              None    => Err("The output of the Constituent Machine 2nd run must not be None".to_string()),
              Some(sm_feedback_val) => Ok((sm_feedback.0,Some(sm_feedback_val)))
            }
          }
        }
      }
    }
  }
  fn step(&mut self, _: &Self::InputType) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,None)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("Feedback::Start state: (SM:{})",self.sm.verbose_state())
  }
  fn verbose_step(&self, inp: &Self::InputType, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Feedback::In: {} Out: None Next State: {}", inp, self.sm.verbose_state()),
      Some(outp) => format!("Feedback::In: {} Out: {} (SM: {})", inp, outp, self.sm.verbose_state())
    }
  }
  fn is_composite(&self) -> bool {
    true
  }

}
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
/*  use accumulator::Accumulator;
  use average2::Average2;*/
  use delay::Delay;
  use increment::Increment;
  use cascade::Cascade;
/*  #[test]
  fn it_cascades_accumulators_next_values() {
    let test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.get_next_values(&(0i8,0i8),Some(&0i8)),Ok(((0i8,0i8),Some(0i8))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,15i8),Some(15i8))));
    assert_eq!(test.get_next_values(&(3i8,5i8),Some(&7i8)),Ok(((10i8,15i8),Some(15i8))));
  }
  #[test]
  fn it_cascades_accumulators_steps() {
    let mut test: Cascade<Accumulator<i8>,Accumulator<i8>> = Cascade::new((1i8,2i8));
    assert_eq!(test.step(&3i8),Ok(6i8));
    assert_eq!(test.state,(4i8,6i8));
    assert_ne!(test.step(&3i8),Ok(6i8));
    assert_ne!(test.state,(4i8,6i8));
  }
  #[test]
  fn it_cascades_average2_next_values() {
    let test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.get_next_values(&(0f64,0f64),Some(&0f64)),Ok(((0f64,0f64),Some(0f64))));
    assert_eq!(test.get_next_values(&(3f64,5f64),Some(&7f64)),Ok(((7f64,5f64),Some(5f64))));
  }
  #[test]
  fn it_cascades_average2_steps() {
    let mut test: Cascade<Average2<f64>,Average2<f64>> = Cascade::new((1f64,2f64));
    assert_eq!(test.step(&3f64),Ok(2f64));
    assert_eq!(test.state,(3f64,2f64));
    assert_eq!(test.step(&2f64),Ok(2.25f64));
    assert_ne!(test.state,(3f64,2f64));
  }
  #[test]
  fn it_cascades_delay_to_increment() {
    let mut test: Cascade<Delay<i64>,Increment<i64>> = Cascade::new((100i64,1i64));
    assert_eq!(test.step(&3i64),Ok(101i64));
    assert_eq!(test.state,(3i64,101i64));
    assert_eq!(test.step(&2i64),Ok(104i64));
    assert_eq!(test.state,(2i64,104i64));
  }*/
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
