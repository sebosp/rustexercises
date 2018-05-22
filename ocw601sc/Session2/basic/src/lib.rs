pub trait StateMachine<StateType=Self> {
  type State;
  fn start(&mut self);
  fn step(&mut self, inp: StateType);
  fn get_next_values<T>(&self, state: StateType, inp: StateType) -> (Self::State,T);
}
pub struct Accumulator {
  pub start_state: i64,
  pub state: i64,
}
impl StateMachine<i64> for Accumulator {
  type State = i64;
  fn start(&mut self) {
    self.state = self.start_state;
  }
  fn step(&mut self, inp: i64) {
    let mut outp:(i64,i64) = self.get_next_values(self.state, inp);
    self.state = outp.0;
  }
  fn get_next_values<i64>(&self, state: i64, inp: i64) -> (i64,i64) {
    (inp + state, inp + state)
  }
}
