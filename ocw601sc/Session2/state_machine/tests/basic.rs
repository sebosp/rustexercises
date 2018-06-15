extern crate state_machine;
#[cfg(test)]
mod tests {
  use state_machine::*;
  use state_machine::accumulator::*;
  use state_machine::gain::*;
  use state_machine::abc::*;
  use state_machine::updown::*;
  use state_machine::delay::*;
  use state_machine::average2::*;
  use state_machine::sumlast3::*;
  use state_machine::selector::*;
  use state_machine::simple_parking_gate::*;
  use state_machine::increment::*;
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
    let mut test_transduce = Average2::new(0f64);
    let transduce_res: Vec<Result<f64,String>> = test_transduce.transduce(vec![100f64,-3f64, 4f64, -123f64, 10f64],true, true);
    assert_eq!(transduce_res, vec![Ok(50f64), Ok(48.5f64), Ok(0.5f64), Ok(-59.5f64), Ok(-56.5f64)]);
  }
  #[test]
  fn test_sumlast3() {
    let mut test_transduce = SumLast3::new((0,0));
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![2,1,3,4,10,1,2,1,5],true, true);
    assert_eq!(transduce_res, vec![Ok(2), Ok(3), Ok(6), Ok(8), Ok(17), Ok(15), Ok(13),Ok(4),Ok(8)]);
  }
  #[test]
  fn test_selector() {
    let max_items:usize = 3;
    let test_next_values = Selector::new(3usize);
    let vec1 = vec![2i64,1i64,3i64,4i64];
    let vec2 = vec![4i64,10i64];
    let next_state1: Result<(usize,Option<Vec<i64>>),String> = test_next_values.get_next_values(&max_items, Some(&vec1));
    assert_eq!(next_state1, Ok((max_items,Some(vec![2i64,1i64, 3i64]))));
    let next_state2: Result<(usize,Option<Vec<i64>>),String> = test_next_values.get_next_values(&max_items, Some(&vec2));
    assert_eq!(next_state2, Err("Requested index out of bounds".to_string()));
    let vec1 = vec!['a','b','.'];
    let vec2 = vec!['y','z'];
    let mut test_transduce = Selector::new(3usize);
    let transduce_res: Vec<Result<Vec<char>,String>> = test_transduce.transduce(vec![vec1,vec2],true, true);
    assert_eq!(transduce_res, vec![Ok(vec!['a','b','.']), Err("Requested index out of bounds".to_string())]);
  }
  #[test]
  fn test_simple_parking_gate() {
    let mut test_transduce = SimpleParkingGate::new(GateState::Waiting);
    let test_input = vec![
      GateSensors{
        car_at_gate: false, car_just_existed: false, position: GatePosition::Bottom
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Bottom
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Bottom
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed:  true, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed:  true, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Top
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Middle
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Bottom
      },
      GateSensors{
        car_at_gate:  true, car_just_existed: false, position: GatePosition::Bottom
      },
    ];
    let transduce_res: Vec<Result<String,String>> = test_transduce.transduce(test_input,true, true);
    assert_eq!(transduce_res, vec![
      Ok("nop".to_string()),
      Ok("raise".to_string()),
      Ok("raise".to_string()),
      Ok("raise".to_string()),
      Ok("raise".to_string()),
      Ok("raise".to_string()),
      Ok("nop".to_string()),
      Ok("nop".to_string()),
      Ok("nop".to_string()),
      Ok("lower".to_string()),
      Ok("lower".to_string()),
      Ok("lower".to_string()),
      Ok("lower".to_string()),
      Ok("lower".to_string()),
      Ok("lower".to_string()),
      Ok("nop".to_string()),
      Ok("raise".to_string()),
    ]);
  }
  #[test]
  fn test_increment() {
    let mut test_transduce = Increment::new(3i64);
    let transduce_res: Vec<Result<i64,String>> = test_transduce.transduce(vec![1i64,2i64,3i64,4i64,5i64],true, true);
    assert_eq!(transduce_res, vec![Ok(4i64), Ok(5i64), Ok(6i64), Ok(7i64), Ok(8i64)]);
  }
}
