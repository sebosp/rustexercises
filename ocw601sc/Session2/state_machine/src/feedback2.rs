//! # Feedback2
use std::fmt::Display;
pub struct Feedback2<SM,T>
  where SM: super::StateMachine,
        SM: super::StateMachine<OutputType=T>,
        SM: super::StateMachine<InputType=super::DualValues<T,T>>,
        <SM>::StateType: Clone + Copy,
        <SM>::InputType: super::DualInput,
        T: Display + Clone + Copy,
{
  pub sm: SM,
  pub state: <SM>::StateType,
}
impl<SM,T> super::StateMachine for Feedback2<SM,T>
  where SM: super::StateMachine,
        SM: super::StateMachine<OutputType=T>,
        SM: super::StateMachine<InputType=super::DualValues<T,T>>,
        <SM>::StateType: Clone + Copy,
        <SM>::InputType: super::DualInput,
        T: Display + Clone + Copy + PartialEq,
{
  /// `StateType`(S) = Something inside constituent SM
  type StateType = <SM>::StateType;
  /// `InputType`(I) = Something inside constituent SM
  type InputType = super::DualValues<T,T>;
  /// `OutputType`(O) = Something inside constituent SM
  type OutputType = <SM>::OutputType;
  fn new(initial_value: Self::StateType) -> Self {
    Feedback2 {
      sm: <SM>::new(initial_value),
      state: initial_value,
    }
  }
  fn start(&mut self){}
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    let sm_next_value = self.sm.get_next_state(&state,inp)?;
    Ok(sm_next_value)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None    => Err("The input of a Feedback2 StateMachine must not be None".to_string()),
      Some(val) => {
        let sm_next_value = self.sm.get_next_values(&state,Some(&super::DualValues{ val1: val.val1, val2: None }))?;
        let sm_feedback   = self.sm.get_next_values(&state,Some(&super::DualValues{ val1: val.val1, val2: sm_next_value.1 }))?;
        match sm_feedback.1 {
          None    => Err("The output of the Constituent Machine Feedback must not be None".to_string()),
          Some(sm_feedback_val) => Ok((sm_feedback.0,Some(sm_feedback_val)))
        }
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    match inp {
      None    => Err("The input of a Feedback2 StateMachine must not be None".to_string()),
      Some(val) => {
        let outp:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,Some(&super::DualValues{ val1: val.val1, val2: None }))?;
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
        let feedback:(Self::StateType,Option<Self::OutputType>) = self.sm.get_next_values(&self.state,Some(&super::DualValues{ val1: val.val1, val2: outp.1 }))?;
        let _ = self.sm.step(Some(&super::DualValues{ val1: val.val1, val2: outp.1 }),verbose,depth+1)?;
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
    }
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}",self.sm.verbose_state(state))
  }
  fn state_machine_name(&self) -> String {
    "Feedback2".to_string()
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None       => format!("In: None"),
      Some(inp)  =>
        match inp.val1 {
          None        => 
            match inp.val2 {
              None        => format!("In: (None,None)"),
              Some(inp_1) => format!("In: (None,{})",inp_1),
            }
          Some(inp_0) => 
            match inp.val2 {
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
    let test = Feedback2::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),None)),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(10i8),None)),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_input_none_some() {
    let test = Feedback2::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(None,Some(10i8))),(10i8,10i8));
  }
  #[test]
  fn it_gets_next_values_some() {
    let test = Feedback2::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(0i8))),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(5i8),Some(7i8))),(12i8,12i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(1i8),Some(0i8))),(1i8,1i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(Some(0i8),Some(1i8))),(1i8,1i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Feedback2::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,Some(0i8))));
    assert_eq!(test.get_next_values(&1i8,None),Ok((1i8,Some(1i8))));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Feedback2::new(0);
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(0i8))),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&(Some(0i8),Some(1i8))),Ok(1i8));
    assert_eq!(test.get_next_state(&5i8,&(Some(3i8),Some(7i8))),Ok(10i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Feedback2::new(0);
    assert_eq!(test.is_composite(),false);
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_cascade() {
    let test: Cascade<Fork<Accumulator<i8>,Accumulator<i8>>,Feedback2<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&5i8),Ok(((7i8,8i8),15i8)));
  }
  #[test]
  fn it_gets_next_state_adder_from_forked_delays() {
    let test: Cascade<Fork<Delay<i8>,Delay<i8>>,Feedback2<i8>> = StateMachine::new(((1i8,2i8),0i8));
    assert_eq!(test.get_next_state(&((0i8, 0i8), 0i8),&0i8),Ok(((0i8,0i8),0i8)));
    assert_eq!(test.get_next_state(&((2i8, 3i8), 0i8),&7i8),Ok(((7i8,7i8),5i8)));
  }
}*/
