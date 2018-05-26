extern crate sm_basics;
#[cfg(test)]
mod tests {
  use sm_basics::*;
  #[test]
  fn test_accumulator() {
    let mut test = Accumulator::new(0);
    test.start();
    assert_eq!(test.step(&10i64),Ok(10i64));
    let mut test_transduce = Accumulator::new(0);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![100i64, -3i64, 4i64, -123i64, 10i64],true, true);
    assert_eq!(transduce_res, vec![Ok(100i64), Ok(97i64), Ok(101i64), Ok(-22i64), Ok(-12i64)]);
  }
  #[test]
  fn test_gain() {
    let mut test_transduce = Gain::new(3);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![10i64, -3i64, 4i64, -123i64, 10i64],true, true);
    assert_eq!(transduce_res, vec![Ok(30i64), Ok(-9i64), Ok(12i64), Ok(-369i64), Ok(30i64)]);
  }
  #[test]
  fn test_abc() {
    let mut test_transduce = ABC::new(0);
    let transduce_res: Vec<Result<bool,String>> = test_transduce.transduce(vec!['a','b','c','a','b'],true, true);
    assert_eq!(transduce_res, vec![Ok(true), Ok(true), Ok(true), Ok(true), Ok(true)]);
    let mut test_transduce2 = ABC::new(0);
    let transduce_res: Vec<Result<bool,String>> = test_transduce2.transduce(vec!['a','a','a'],true, true);
    assert_eq!(transduce_res, vec![Ok(true), Ok(false), Ok(false)]);
  }
  #[test]
  fn test_updown() {
    let mut test_transduce = UpDown::new(0);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec!['u','u','u','d','d','u'],true, true);
    assert_eq!(transduce_res, vec![Ok(1), Ok(2), Ok(3), Ok(2), Ok(1), Ok(2)]);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec!['o'],true, true);
    assert_eq!(transduce_res, vec![Err("Invalid char for UpDown".to_string())]);
  }
  #[test]
  fn test_delay() {
    let mut test_transduce = Delay::new(7);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![3,1,2,5,9],true, true);
    assert_eq!(transduce_res, vec![Ok(7), Ok(3), Ok(1), Ok(2), Ok(5)]);
    let mut test_transduce2 = Delay::new(100);
    let transduce_res: Vec<Result<i64,String>> = test_transduce2.transduce(vec![3,1,2,5,9],true, true);
    assert_eq!(transduce_res, vec![Ok(100), Ok(3), Ok(1), Ok(2), Ok(5)]);
  }
  #[test]
  fn test_average2() {
    let mut test_transduce = Average2::new(0);
    let transduce_res: Vec<Result<f64,String>> = test_transduce.transduce(vec![100,-3, 4, -123, 10],true, true);
    assert_eq!(transduce_res, vec![Ok(50f64), Ok(48.5f64), Ok(0.5f64), Ok(-59.5f64), Ok(-56.5f64)]);
  }
}
