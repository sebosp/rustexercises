//! # Adder
//! A very simple machine who's input is two numbers, it returns the sum.
extern crate num_traits;
use num_traits::*;
// There is no reason for a struct to exist in this StateMachine.
pub struct Adder<T>
where T: Num + Clone + Copy,
{
  pub state: T,
}
impl<T> super::StateMachine for Adder<T>
where T: Num + Clone + Copy,
{
  /// `StateType`(S) = numbers
  type StateType = T;
  /// `InputType`(I) = numbers
  type InputType = (T,T);
  /// `OutputType`(O) = numbers
  type OutputType = T;
  fn new(_: Self::StateType) -> Self {
    Adder {
      state: T::zero(),
    }
  }
  fn start(&mut self){
    self.state = T::zero()
  }
  fn get_next_state(&self, _: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    let inp = *inp;
    Ok(inp.0 + inp.1)
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        // XXX: Check for infinity.
        Ok((next_state,Some(next_state)))
      }
    }
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Option<Self::OutputType>, String> {
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,Some(inp))?;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
    format!("Adder::No Start state")
  }
  fn verbose_step(&self, _: &Self::InputType, _: Option<&Self::OutputType>) -> String {
    //format!("In: ({},{}) Out: {} Next State: {}", inp.0, inp.1, outp, self.state) # XXX: FIXME
    format!("Adder::In: XXX Out: XXX Next State: XXX")
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_some() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(0i8,0i8)),(0i8,0i8));
    assert_eq!(test.get_next_values_wrap_unwrap(&0i8,&(5i8,7i8)),(12i8,12i8));
  }
  #[test]
  fn it_gets_next_values_none() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_values(&0i8,None),Ok((0i8,None)));
    assert_eq!(test.get_next_values(&1i8,None),Ok((1i8,None)));
  }
  #[test]
  fn it_gets_next_state() {
    let test = Adder::new(0);
    assert_eq!(test.get_next_state(&0i8,&(0i8,0i8)),Ok(0i8));
    assert_eq!(test.get_next_state(&0i8,&(0i8,1i8)),Ok(1i8));
    assert_eq!(test.get_next_state(&5i8,&(3i8,7i8)),Ok(10i8));
  }
  #[test]
  fn it_checks_is_composite() {
    let test = Adder::new(0);
    assert_eq!(test.is_composite(),false);
  }
}
