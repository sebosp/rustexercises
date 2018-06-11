extern crate state_machine;
#[cfg(test)]
mod tests {
  use state_machine::*;
  use state_machine::cascade::*;
//  use state_machine::accumulator::*;
//  use state_machine::gain::*;
//  use state_machine::abc::*;
//  use state_machine::updown::*;
  use state_machine::delay::*;
//  use state_machine::average2::*;
//  use state_machine::sumlast3::*;
//  use state_machine::selector::*;
//  use state_machine::simple_parking_gate::*;
  use state_machine::increment::*;
  #[test]
  fn test_cascade_delay() {
    let mut cascade: Cascade<Delay<i64>,Delay<i64>> = Cascade::new((99i64,22i64));
    let transduce_res: Vec<Result<i64,String>> = cascade.transduce(vec![3i64,8i64,2i64,4i64,6i64,5i64],true, true);
    assert_eq!(transduce_res, vec![Ok(22i64), Ok(99i64), Ok(3i64), Ok(8i64), Ok(2i64), Ok(4i64)]);
  }
  #[test]
  fn test_increment() {
    let mut test_transduce = Increment::new(3i64);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![1i64,2i64,3i64,4i64,5i64],true, true);
    assert_eq!(transduce_res, vec![Ok(4i64), Ok(5i64), Ok(6i64), Ok(7i64), Ok(8i64)]);
  }
}
