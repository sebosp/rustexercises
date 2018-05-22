extern crate sm_basics;
#[cfg(test)]
mod tests {
  use sm_basics::*;
  #[test]
  fn test_accumulator() {
    let mut test = Accumulator {
      state: 0i64,
      start_state: 0i64
    };
    test.start();
    test.step(10i64);
    assert_eq!(test.state,10i64);
  }
}
