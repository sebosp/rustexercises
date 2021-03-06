//! # SimpleParkingGate
//! Simple state machine that implements the controller for the parking gate.
//! The machine has three sensors:
//! - gatePosition: has one of three values signifying the position of the arm
//!                 of the parking gate: 'top', 'middle', 'bottom'
//!
//! - carAtGate: true if a car is waiting to come through the gate and false
//!              otherwise.
//!
//! - carJustExited: true if a car has just passed through the gate; it is true
//!                  for only one step before resetting to False. 
//! The machine has three possible outputs:
//! - 'raise'
//! - 'lower'
//! - 'nop' (no operation) -> None
//!
//! The machine has four possible states:
//! - 'waiting'  (for a car to arrive at the gate),
//! - 'raising'  (the arm),
//! - 'raised'   (the arm is at the top position and we're waiting for the car to
//!            drive through the gate), 
//! - 'lowering' (the arm). 
#[derive(PartialEq, Clone, Copy)]
pub enum GatePosition {
  Top,
  Middle,
  Bottom
}
#[derive(PartialEq, Clone, Copy)]
pub struct GateSensors {
  pub position: GatePosition,
  pub car_at_gate: bool,
  pub car_just_existed: bool
}
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GateState {
  Waiting,
  Raising,
  Raised,
  Lowering
}
pub struct SimpleParkingGate {
  pub state: GateState,
}
impl super::StateMachine for SimpleParkingGate {
  /// `StateType`(S) = number
  type StateType = GateState;
  /// `InputType`(I) = char
  type InputType = GateSensors;
  /// `OutputType`(O) = number
  type OutputType = String;
  /// `initial_value`(_s0_) is usually zero
  fn new(initial_value: Self::StateType) -> Self {
    SimpleParkingGate {
      state: initial_value
    }
  }
  fn start(&mut self){
    self.state = GateState::Waiting;
  }
  fn get_next_state(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<Self::StateType, String> {
    match state {
      GateState::Waiting => {
        if inp.car_at_gate {
          Ok(GateState::Raising)
        } else {
          if inp.position != GatePosition::Bottom {
            Err("GatePosition and GateState sensors have invalid data".to_string())
          } else {
          Ok(GateState::Waiting)
          }
        }
      },
      GateState::Raising => {
        if inp.position == GatePosition::Top {
          Ok(GateState::Raised)
        } else {
          Ok(GateState::Raising)
        }
      },
      GateState::Raised => {
        if inp.car_just_existed {
          Ok(GateState::Lowering)
        } else {
          if inp.position != GatePosition::Top {
            Err("GatePosition and GateState sensors have invalid data".to_string())
          } else {
            Ok(GateState::Raised)
          }
        }
      },
      GateState::Lowering => {
        if inp.position == GatePosition::Bottom {
          Ok(GateState::Waiting)
        } else {
          Ok(GateState::Lowering)
        }
      }
    }
  }
  fn get_next_values(&self, state: &Self::StateType, inp: Option<&Self::InputType>) -> Result<(Self::StateType,Option<Self::OutputType>),String> {
    match inp {
      None => Ok((*state,None)),
      Some(inp) => {
        let next_state = self.get_next_state(state,inp)?;
        Ok((next_state,self.output_state(next_state)))
      }
    }
  }
  fn step(&mut self, inp: Option<&Self::InputType>, verbose: bool, depth: usize) -> Result<Option<Self::OutputType>, String> {
    //let temp_inp = inp;
    let outp:(Self::StateType,Option<Self::OutputType>) = self.get_next_values(&self.state,inp)?;
    if verbose {
      println!("{}{}::{} {} -> ({},{})",
             "  ".repeat(depth),
             self.state_machine_name(),
             self.verbose_state(&self.state),
             self.verbose_input(inp),
             self.verbose_state(&outp.0),
             self.verbose_output(outp.1.as_ref()))
    }
    self.state = outp.0;
    match outp.1 {
      None           => Ok(Some("nop".to_string())),
      Some(next_val) => Ok(Some(next_val))
    }
  }
  fn state_machine_name(&self) -> String {
    "SimpleParkingGate".to_string()
  }
  fn verbose_output(&self, outp: Option<&Self::OutputType>) -> String {
    match outp {
      None       => format!("Out: {}","nop".to_string()),
      Some(outp) => format!("Out: {}",outp)
    }
  }
  fn verbose_input(&self, inp: Option<&Self::InputType>) -> String {
    match inp {
      None      => "None".to_string(),
      Some(inp) => {
        let gate_position: String = match inp.position {
          GatePosition::Top    => "Top".to_string(),
          GatePosition::Middle => "Middle".to_string(),
          GatePosition::Bottom => "Bottom".to_string(),
        };
        format!("In: (Car At Gate: {}, Car Just Exited: {}, Gate Position: {})",inp.car_at_gate,inp.car_just_existed,gate_position)
      }
    }
  }
  fn verbose_state(&self, state: &Self::StateType) -> String {
    match state {
      GateState::Waiting  => "State: Waiting".to_string(),
      GateState::Raising  => "State: Raising".to_string(),
      GateState::Raised   => "State: Raised".to_string(),
      GateState::Lowering => "State: Lowering".to_string(),
    }
  }
  fn get_state(&self) -> Self::StateType{
    self.state
  }
}
impl SimpleParkingGate {
  fn output_state(&self, state: GateState) -> Option<String> {
    match state {
      GateState::Raising  => Some("raise".to_string()),
      GateState::Lowering => Some("lower".to_string()),
      _                   => None,
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_outputs_state() {
    let test = SimpleParkingGate::new(GateState::Waiting);
    assert_eq!(test.output_state(GateState::Waiting),  None);
    assert_eq!(test.output_state(GateState::Raising),  Some("raise".to_string()));
    assert_eq!(test.output_state(GateState::Lowering), Some("lower".to_string()));
  }
  #[test]
  fn it_gets_next_values_gate_down_no_car() {
    let test = SimpleParkingGate::new(GateState::Waiting);
    // GatePosition::Bottom
    assert_eq!(
      test.get_next_values( // Cannot be wrapped because of None
        &GateState::Waiting,
        Some(&GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        })
      ),Ok((GateState::Waiting,None))
    );
    assert_eq!(
      test.get_next_values_wrap_unwrap(
        &GateState::Raising,
        &GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        }
      ),(GateState::Raising,"raise".to_string())
    );
    assert_eq!(
      test.get_next_values( // Cannot be wrapped because of Err()
        &GateState::Raised,
        Some(&GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        })
      ),Err("GatePosition and GateState sensors have invalid data".to_string())
    );
    assert_eq!(
      test.get_next_values( // Cannot be wrapped because of None
        &GateState::Lowering,
        Some(&GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        })
      ),Ok((GateState::Waiting,None))
    );
  }
  #[test]
  fn it_checks_is_composite() {
    let test = SimpleParkingGate::new(GateState::Waiting);
    assert_eq!(test.is_composite(),false);
  }
}
