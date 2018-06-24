//! # ABC State Machine
//! This is a finite-state machine whose output is true if the input string
//! adheres to a simple pattern, and false otherwise. In this case,
//! the pattern has to be a, b, c, a, b, c, a, b, c, . . .. 
pub struct ABC {
  pub state: i8,
}
impl super::StateMachine for ABC {
  /// `StateType`(S) is a number from 0 to 3
  type StateType = i8;
  /// `InputType`(I) is a, b, or c. Any other char returns an Err()
  type InputType = char;
  /// `OutputType`(O) is either true or false
  type OutputType = bool;
  /// Returns an ABC struct. `initial_value`(_s0_) is usually 0.
  fn new(initial_value: Self::StateType) -> Self {
    ABC {
      state: initial_value,
    }
  }
  fn start(&mut self){
    self.state = Self::StateType::from(0);
  }
  fn get_next_state(&self, _: &Self::StateType, _: &Self::InputType) -> Result<Self::StateType, String> {
    Ok(Self::StateType::from(0))
  }
  /// ABC uses the states 0, 1, and 2 to stand for the situations in which it
  /// is expecting an a, b, and c, respectively; and it uses state 3 for the 
  /// situation in which it has seen an input that was not the one that was
  /// expected. Once the machine goes to state 3 (sometimes called a rejecting
  /// state), it never exits that state. 
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None      => Ok((*state,None)),
      Some(inp) => {
        let state = *state;
        let inp = *inp;
        if state == 0 && inp == 'a' {
          Ok((Self::StateType::from(1), Some(true)))
        } else if state == 1 && inp == 'b' {
          Ok((Self::StateType::from(2), Some(true)))
        } else if state == 2 && inp == 'c' {
          Ok((Self::StateType::from(0), Some(true)))
        } else if inp != 'a' && inp != 'b' && inp != 'c' {
          Err("Unsupported character".to_string())
        }else {
          Ok((Self::StateType::from(3), Some(false)))
        }
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: i8) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(self.state),
             self.verbose_input(inp),
             self.verbose_state(outp.0),
             self.verbose_output(outp.1))
    }
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    format!("State: {}", state)
  }
  fn state_machine_name(&self) -> String {
    "ABC".to_string()
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
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_good_seq() {
    let test = ABC::new(0);
    assert_eq!(test.get_next_values(&0i8,Some(&'a')),Ok((1i8,Some(true))));
    assert_eq!(test.get_next_values(&1i8,Some(&'b')),Ok((2i8,Some(true))));
    assert_eq!(test.get_next_values(&2i8,Some(&'c')),Ok((0i8,Some(true))));
    assert_eq!(test.get_next_values(&0i8,Some(&'a')),Ok((1i8,Some(true))));
    assert_eq!(test.get_next_values(&1i8,Some(&'b')),Ok((2i8,Some(true))));
    assert_eq!(test.get_next_values(&2i8,Some(&'c')),Ok((0i8,Some(true))));
  }
  #[test]
  fn it_gets_next_values_bad_seq() {
    let test = ABC::new(0);
    assert_eq!(test.get_next_values(&2i8,Some(&'b')),Ok((3i8,Some(false))));
  }
  #[test]
  fn it_gets_next_values_bad_char() {
    let test = ABC::new(0);
    assert_eq!(test.get_next_values(&2i8,Some(&'d')),Err("Unsupported character".to_string()));
  }
  #[test]
  fn it_steps_good_seq() {
    let mut test = ABC::new(0);
    assert_eq!(test.step_unwrap(&'a'),true);
    assert_eq!(test.state,1);
  }
  #[test]
  fn it_steps_bad_seq() {
    let mut test = ABC::new(0);
    assert_eq!(test.step_unwrap(&'a'),true);
    assert_eq!(test.step_unwrap(&'a'),false);
    assert_eq!(test.step_unwrap(&'a'),false);
    assert_eq!(test.step_unwrap(&'a'),false);
    assert_eq!(test.step_unwrap(&'a'),false);
    assert_eq!(test.state,3);
  }
  #[test]
  fn it_gets_next_state() {
    let test = ABC::new(0);
    assert_eq!(test.get_next_state(&0i8,&'a'),Ok(0i8));
    assert_eq!(test.get_next_state(&1i8,&'b'),Ok(0i8));
    assert_eq!(test.get_next_state(&2i8,&'c'),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&'a'),Ok(0i8));
    assert_eq!(test.get_next_state(&1i8,&'b'),Ok(0i8));
    assert_eq!(test.get_next_state(&2i8,&'c'),Ok(0i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = ABC::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
