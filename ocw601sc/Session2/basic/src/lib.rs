pub trait StateMachine<StateType=Self> {
  type State;
  type InputType;
  type OutputType;
  fn new(initial_value: StateType) -> Self;
  fn start(&mut self);
  fn step(&mut self, inp: Self::InputType) -> Self::OutputType;
  fn get_next_state(&self, state: StateType, inp: Self::InputType) -> Self::State;
  fn get_next_values(&self, state: StateType, inp: Self::InputType) -> (Self::State,Self::OutputType);
  fn transduce(&mut self, inp: Vec<Self::InputType>, verbose: bool, compact: bool) -> Vec<Result<Self::OutputType, String>>;
}
pub struct Accumulator {
  pub start_state: i64,
  pub state: i64,
}
impl StateMachine<i64> for Accumulator {
  type State = i64;
  type InputType = i64;
  type OutputType = i64;
  fn new(initial_value: i64) -> Self{
    Accumulator {
      start_state: initial_value,
      state: initial_value
    }
  }
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn step(&mut self, inp: i64) -> i64 {
    let outp:(i64,i64) = self.get_next_values(self.state, inp);
    self.state = outp.0;
    outp.1
  }
  fn get_next_state(&self, state: i64, inp: i64) -> i64 {
    inp + state
  }
  fn get_next_values(&self, state: i64, inp: i64) -> (i64,i64) {
    let next_state = self.get_next_state(state,inp);
    (next_state,next_state)
  }
  fn transduce(&mut self, inp: Vec<i64>, verbose: bool, _: bool) -> Vec<Result<i64, String>> {
    let mut res: Vec<Result<i64, String>> = Vec::new();
    if verbose {
      println!("Accumulator Start state: {}", self.state);
    }
    for cur_inp in inp {
      let cur_out = self.step(cur_inp);
      if verbose {
        println!("In: {} Out: {} Next State: {}", cur_inp, cur_out, self.state);
      }
      res.push(Ok(self.state));
    }
    res
  }
}
pub struct Gain {
  pub k: i64,
}
impl StateMachine<i64> for Gain {
  type State = i64;
  type InputType = i64;
  type OutputType = i64;
  fn new(initial_value: i64) -> Self{
    Gain {
      k: initial_value
    }
  }
  fn start(&mut self){}
  fn step(&mut self, inp: i64) -> i64 {
    let outp:(i64,i64) = self.get_next_values(0i64,inp);
    outp.1
  }
  fn get_next_state(&self, _: i64, inp: i64) -> i64 {
    inp * self.k
  }
  fn get_next_values(&self, unused: i64, inp: i64) -> (i64,i64) {
    let next_state = self.get_next_state(unused,inp);
    (next_state,next_state)
  }
  fn transduce(&mut self, inp: Vec<i64>, verbose: bool, _: bool) -> Vec<Result<i64, String>> {
    let mut res: Vec<Result<i64, String>> = Vec::new();
    for cur_inp in inp {
      let cur_out = self.step(cur_inp);
      if verbose {
        println!("Gain In: {} Out: {} Next State: {}", cur_inp, cur_out, self.k);
      }
      res.push(Ok(cur_out));
    }
    res
  }
}
pub struct ABC {
  pub state: i8,
}
impl StateMachine<i8> for ABC {
  type State = i8;
  type InputType = char;
  type OutputType = bool;
  fn new(initial_value: i8) -> Self {
    ABC {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = 0i8;
  }
  fn step(&mut self, inp: char) -> bool {
    let outp:(i8,bool) = self.get_next_values(self.state,inp);
    self.state = outp.0;
    outp.1
  }
  fn get_next_state(&self, _: i8, _: char) -> i8 {
    0i8
  }
  fn get_next_values(&self, state: i8, inp: char) -> (i8,bool) {
    if state == 0 && inp == 'a' {
      (1i8, true)
    } else if state == 1 && inp == 'b' {
      (2i8, true)
    } else if state == 2 && inp == 'c' {
      (0i8, true)
    } else {
      (3i8, false)
    }
  }
  fn transduce(&mut self, inp: Vec<char>, verbose: bool, _: bool) -> Vec<Result<bool, String>> {
    let mut res: Vec<Result<bool, String>> = Vec::new();
    for cur_inp in inp {
      let cur_out = self.step(cur_inp);
      if verbose {
        println!("ABC In: {} Out: {} Next State: {}", cur_inp, cur_out, self.state);
      }
      res.push(Ok(cur_out));
    }
    res
  }
}
