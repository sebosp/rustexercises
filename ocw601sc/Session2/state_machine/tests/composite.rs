extern crate state_machine;
#[cfg(test)]
mod tests {
  use state_machine::*;
  use state_machine::cascade::*;
  use state_machine::adder::*;
  use state_machine::wire::*;
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
  use state_machine::feedback::*;
  use state_machine::fork::*;
  #[test]
  fn test_cascade_delay() {
    let mut cascade: Cascade<Delay<i64>,Delay<i64>> = Cascade::new((99i64,22i64));
    let transduce_res: Vec<Result<i64,String>> = cascade.transduce_wrap_unwrap(vec![3i64,8i64,2i64,4i64,6i64,5i64],true, true);
    assert_eq!(transduce_res, vec![Ok(22i64), Ok(99i64), Ok(3i64), Ok(8i64), Ok(2i64), Ok(4i64)]);
  }
  #[test]
  fn it_feedbacks_cascades_increment_to_delay() {
    let mut feedback: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((2i64,3i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(3i64)), Ok(Some(5i64)), Ok(Some(7i64)), Ok(Some(9i64)), Ok(Some(11i64)), Ok(Some(13i64))]);
  }
  #[test]
  fn it_feedbacks_cascades_delay_to_increment45() {
    let mut feedback: Feedback<Cascade<Delay<i64>,Increment<i64>>> = StateMachine::new((1i64,1i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(4i64)), Ok(Some(5i64)), Ok(Some(6i64)), Ok(Some(7i64))]);
  }
  #[test]
  fn it_feedbacks_cascades_increment_to_delay45() {
    let mut feedback: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((1i64,1i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(4i64)), Ok(Some(5i64)), Ok(Some(6i64))]);
  }
  #[test]
  fn it_cascades_fork_delays_adder() {
    let mut feedback:
          Cascade<
            Fork<
              Delay<i64>,
              Cascade<
                Delay<i64>,
                Delay<i64>
              >
            >,
            Adder<i64>
          > = StateMachine::new(((1i64,(1i64, 0i64)),0i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(2i64)), Ok(Some(2i64)), Ok(Some(2i64)), Ok(Some(2i64))]);
  }
  /// Fibonacci sequence using Feedback, Cascade, Fork and 3 delays.
  ///
  /// Feedback
  ///             ............................................. ...........
  ///             :            -----                          : :         :
  ///             :        /->|Delay|------------------------>:-:---      :
  ///     ------- :  ----  |   -----                          : :   |     :
  ///  ->|Cascade|->|Fork|-|           .......... ........... : :   v     :
  /// |   ------- :  ----  |   ------- :  ----- : :  -----  : : :  -----  :
  /// |           :        \->|Cascade|->|Delay|--->|Delay|-->:-:>|Adder|-:-
  /// |           :            ------- :  ----- : :  -----  : : :  -----  : |
  /// |           :                    :........: :.........: : :         : v
  /// |           :                                           : :         : |
  /// |           :...........................................: :.........: |
  /// |                                                                     |
  ///  ---<------------------------------------------------------------<----
  #[test]
  fn it_fibonaccis() {
    let mut feedback:
        Feedback<
          Cascade<
            Fork<
              Delay<i64>,
              Cascade<
                Delay<i64>,
                Delay<i64>
              >
            >,
            Adder<i64>
          >
        > = StateMachine::new(((1i64,(1i64, 0i64)),0i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(5i64)), Ok(Some(8i64)), Ok(Some(13i64))]);
  }
  #[test]
  fn it_fibonaccis_good_start() {
    // Exercise 4.6
    let mut feedback:
        Feedback<
          Cascade<
            Fork<
              Delay<i64>,
              Cascade<
                Delay<i64>,
                Delay<i64>
              >
            >,
            Adder<i64>
          >
        > = StateMachine::new(((0i64,(0i64, 1i64)),0i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(5i64)), Ok(Some(8i64))]);
  }
  #[test]
  fn it_fibonaccis_two_delays_with_wire47() {
    // Exercise 4.7
    let mut feedback:
        Feedback<
          Cascade<
            Fork<
              Wire<i64>,
              Cascade<
                Delay<i64>,
                Delay<i64>
              >
            >,
            Adder<i64>
          >
        > = StateMachine::new(((0i64,(0i64, 1i64)),0i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(5i64)), Ok(Some(8i64))]);
  }
}
