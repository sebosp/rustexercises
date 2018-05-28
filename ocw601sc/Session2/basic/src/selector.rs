//! # Selector
//! A simple functional machine that is very useful is the Select machine. You
//! can make many different versions of this, but the simplest one takes an 
//! input that is a stream of lists or tuples of several values (or structures
//! of values) and generates the stream made up only of the kth elements of
//! the input values. 
pub struct Selector {
  pub k: usize,
}
impl super::StateMachine for Selector {
  /// `StateType`(S) = slice size to return
  type StateType = usize;
  /// `InputType`(I) = Vector of numbers
  type InputType = Vec<i64>;
  /// `OutputType`(O) = Vector of numbers
  type OutputType = Vec<i64>;
  /// `num_elements`(k)
  fn new(num_elements: Self::StateType) -> Self {
    Selector {
      k: num_elements
    }
  }
  fn start(&mut self){}
  fn step(&mut self, _: &Self::InputType) -> Result<Self::OutputType, String> {
    Err("Selector does not implement step() function".to_string())
  }
  fn get_next_state(&self, _: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    if self.k > inp.len() {
      Err("Requested index out of bounds".to_string())
    } else {
      Ok(self.k)
    }
  }
  fn get_next_values(&self, unused: Self::StateType, mut inp: Self::InputType) -> Result<(Self::StateType, Self::OutputType),String> {
    // Might be expensive to clone the Vector if it's big.
    let next_state = self.get_next_state(unused,inp.to_owned())?;
    inp.truncate(self.k);
    Ok((next_state, inp))
  }
  fn verbose_state(&self) -> String {
     format!("Selector k: ({:?})",self.k)
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {:?} Out: {:?}", inp, outp)
  }
}
