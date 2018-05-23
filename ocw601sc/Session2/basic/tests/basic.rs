extern crate sm_basics;
#[cfg(test)]
mod tests {
  use sm_basics::*;
  #[test]
  fn test_accumulator() {
    let mut test = Accumulator::new(0);
    test.start();
    test.step(10i64);
    assert_eq!(test.state,10i64);
    let mut test_transduce = Accumulator::new(0);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![100i64, -3i64, 4i64, -123i64, 10i64],true, true);
    assert_eq!(transduce_res, vec![Ok(100i64), Ok(97i64), Ok(101i64), Ok(-22i64), Ok(-12i64)]);
  }
}
