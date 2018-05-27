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
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = Self::StateType::from(0);
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn get_next_state(&self, _: Self::StateType, _: Self::InputType) -> Result<Self::StateType, String> {
    Ok(Self::StateType::from(0))
  }
  /// ABC uses the states 0, 1, and 2 to stand for the situations in which it
  /// is expecting an a, b, and c, respectively; and it uses state 3 for the 
  /// situation in which it has seen an input that was not the one that was
  /// expected. Once the machine goes to state 3 (sometimes called a rejecting
  /// state), it never exits that state. 
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    if state == 0 && inp == 'a' {
      Ok((Self::StateType::from(1), true))
    } else if state == 1 && inp == 'b' {
      Ok((Self::StateType::from(2), true))
    } else if state == 2 && inp == 'c' {
      Ok((Self::StateType::from(0), true))
    } else if inp != 'a' && inp != 'b' && inp != 'c' {
      Err("Unsupported character".to_string())
    }else {
      Ok((Self::StateType::from(3), false))
    }
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
