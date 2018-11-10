extern crate state_machine;
#[cfg(test)]
mod tests {
  use state_machine::*;
  use state_machine::cascade::*;
  use state_machine::adder::*;
  use state_machine::wire::*;
//  use state_machine::accumulator::*;
  use state_machine::gain::*;
  use state_machine::negation::*;
//  use state_machine::abc::*;
//  use state_machine::updown::*;
  use state_machine::delay::*;
//  use state_machine::average2::*;
//  use state_machine::sumlast3::*;
//  use state_machine::selector::*;
//  use state_machine::simple_parking_gate::*;
  use state_machine::increment::*;
  use state_machine::multiplier::*;
  use state_machine::feedback::*;
  use state_machine::fork::*;
  #[test]
  fn test_cascade_delay() {
    let mut cascade = CascadeBuilder::new()
      .with_src(Delay::new(99i64))
      .with_dst(Delay::new(22i64))
      .build().unwrap();
    let transduce_res: Vec<Result<i64,String>> = cascade.transduce_wrap_unwrap(vec![3i64,8i64,2i64,4i64,6i64,5i64],true, true);
    assert_eq!(transduce_res, vec![Ok(22i64), Ok(99i64), Ok(3i64), Ok(8i64), Ok(2i64), Ok(4i64)]);
  }
  #[test]
  fn it_feedbacks_cascades_increment_to_delay() {
    let mut feedback = FeedbackBuilder::new()
      .with_inner(
        CascadeBuilder::new()
          .with_src(Increment::new(2i64))
          .with_dst(Delay::new(3i64))
          .build().unwrap()
      )
      .build().unwrap();
    //let mut feedback: Feedback<Cascade<Increment<i64>,Delay<i64>>> = StateMachine::new((2i64,3i64));
    //let mut feedback: Feedback<_> = StateMachine::new((2i64,3i64));
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
  fn it_feedbacks_negation_alternates() {
    // Exercise 4.4
    let mut feedback: Feedback<Negation> = StateMachine::new(true);
    let transduce_res: Vec<Result<Option<bool>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(false)),Ok(Some(true)), Ok(Some(false)), Ok(Some(true)), Ok(Some(false)), Ok(Some(true))]);
  }
  #[test]
  fn it_cascades_fork_delays_adder() {
    let mut test:
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
    let transduce_res: Vec<Result<Option<i64>,String>> = test.transduce(vec![None, None, None, None, None, None],true, true);
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
            Delay<i64>,
            Cascade<
              Fork<
                Wire<i64>,
                Delay<i64>
              >,
              Adder<i64>,
            >
          >
        > = StateMachine::new((1i64,((1i64, 0i64),0i64)));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(1i64)), Ok(Some(2i64)),Ok(Some(3i64)), Ok(Some(5i64)), Ok(Some(8i64)), Ok(Some(13i64))]);
  }
  #[test]
  fn it_feedback_doubles_gain48() {
    // Exercise 4.8
    let mut feedback:
        Feedback<
          Cascade<
            Delay<i64>,
            Gain<i64>,
          >
        > = StateMachine::new((1i64,2i64));
    let transduce_res: Vec<Result<Option<i64>,String>> = feedback.transduce(vec![None, None, None, None, None, None],true, true);
    assert_eq!(transduce_res, vec![Ok(Some(2i64)),Ok(Some(4i64)), Ok(Some(8i64)), Ok(Some(16i64)), Ok(Some(32i64)), Ok(Some(64i64))]);
  }
  #[test]
  fn it_feedback_squares49() {
    // Exercise 4.9
    let mut feedback2:
        Feedback<
          Cascade<
            Fork<
              Delay<i64>,
              Delay<i64>,
            >,
            Multiplier<i64>
          >
        > = StateMachine::new(((1i64,2i64),0i64));
    let transduce_res2: Vec<Result<Option<i64>,String>> = feedback2.transduce(vec![None, None, None, None, None],true, true);
    assert_eq!(transduce_res2, vec![Ok(Some(2i64)),Ok(Some(4i64)), Ok(Some(16i64)), Ok(Some(256i64)), Ok(Some(65536i64))]);
    let mut feedback3:
        Feedback<
          Cascade<
            Fork<
              Delay<i64>,
              Delay<i64>,
            >,
            Multiplier<i64>
          >
        > = StateMachine::new(((1i64,3i64),0i64));
    let transduce_res3: Vec<Result<Option<i64>,String>> = feedback3.transduce(vec![None, None, None, None],true, true);
    assert_eq!(transduce_res3, vec![Ok(Some(3i64)),Ok(Some(9i64)), Ok(Some(81i64)), Ok(Some(6561i64))]);
  }
}
