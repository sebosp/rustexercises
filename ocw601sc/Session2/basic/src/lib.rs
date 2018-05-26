pub trait StateMachine {
  type StateType;
  type InputType;
  type OutputType;
  fn new(initial_value: Self::StateType) -> Self;
  fn start(&mut self);
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String>;
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String>;
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String>;
  fn verbose_state(&self) -> String;
  fn verbose_step(&self, inp: &Self::InputType, outp: &Self::OutputType) -> String;
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
pub struct Accumulator {
  pub start_state: i64,
  pub state: i64,
}
impl StateMachine for Accumulator {
  type StateType = i64;
  type InputType = i64;
  type OutputType = i64;
  fn new(initial_value: Self::StateType) -> Self {
    Accumulator {
      start_state: initial_value,
      state: initial_value
    }
  }
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp + state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
pub struct Gain {
  pub k: i64,
}
impl StateMachine for Gain {
  type StateType = i64;
  type InputType = i64;
  type OutputType = i64;
  fn new(initial_value: Self::StateType) -> Self {
    Gain {
      k: initial_value
    }
  }
  fn start(&mut self){}
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(Self::StateType::from(0),*inp)?;
    Ok(outp.1)
  }
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    Ok(inp * self.k)
  }
  fn get_next_values(&self, unused: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(unused,inp)?;
    Ok((next_state,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Gain K: {}",self.k)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.k)
  }
}
pub struct ABC {
  pub state: i8,
}
impl StateMachine for ABC {
  type StateType = i8;
  type InputType = char;
  type OutputType = bool;
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
pub struct UpDown {
  pub state: i64,
}
impl StateMachine for UpDown {
  type StateType = i64;
  type InputType = char;
  type OutputType = i64;
  fn new(initial_value: Self::StateType) -> Self {
    UpDown {
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
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    if inp == 'u' {
      Ok(state + Self::StateType::from(1))
    } else  if inp == 'd' {
      Ok(state - Self::StateType::from(1))
    } else {
      Err("Invalid direction for UpDown".to_string())
    }
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
pub struct Delay {
  pub state: i64,
}
impl StateMachine for Delay {
  type StateType = i64;
  type InputType = i64;
  type OutputType = i64;
  fn new(initial_value: Self::StateType) -> Self {
    Delay {
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
  fn get_next_state(&self, state: Self::StateType, _: Self::InputType) -> Result<Self::StateType, String> {
    Ok(state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((inp,next_state))
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.state)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", inp, outp, self.state)
  }
}
