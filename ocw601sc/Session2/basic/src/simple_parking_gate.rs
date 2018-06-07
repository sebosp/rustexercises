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
//! - 'nop' (no operation)
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
  fn get_next_state(&self, state: Self::StateType, inp: Self::InputType) -> Result<Self::StateType, String> {
    let mut next_state = state;
    match state {
      GateState::Waiting => {
        if inp.car_at_gate {
          next_state = GateState::Raising;
        } else if inp.position != GatePosition::Bottom {
          return Err("GatePosition and GateState sensors have invalid data".to_string());
        }
      },
      GateState::Raising => {
        if inp.position == GatePosition::Top {
          next_state = GateState::Raised;
        }
      },
      GateState::Raised => {
        if inp.car_just_existed {
          next_state = GateState::Lowering;
        } else if inp.position != GatePosition::Top {
          return Err("GatePosition and GateState sensors have invalid data".to_string());
        }
      },
      GateState::Lowering => {
        if inp.position == GatePosition::Bottom {
          next_state = GateState::Waiting;
        }
      },
    };
    Ok(next_state)
  }
  fn get_next_values(&self, state: Self::StateType, inp: Self::InputType) -> Result<(Self::StateType,Self::OutputType),String> {
    let next_state = self.get_next_state(state,inp)?;
    Ok((next_state,self.verbose_output(next_state)))
  }
  fn step(&mut self, inp: &Self::InputType) -> Result<Self::OutputType, String> {
    let temp_inp = inp;
    let outp:(Self::StateType,Self::OutputType) = self.get_next_values(self.state,*temp_inp)?;
    self.state = outp.0;
    Ok(outp.1)
  }
  fn verbose_state(&self) -> String {
     format!("Start state: {}",self.verbose_output(self.state))
  }
  fn verbose_step(&self,inp: &Self::InputType, outp: &Self::OutputType) -> String {
     format!("In: {} Out: {} Next State: {}", self.verbose_input(inp), outp, self.verbose_state())
  }
}
impl SimpleParkingGate {
  fn verbose_output(&self, state: GateState) -> String {
    match state {
      GateState::Raising => {
        "raise".to_string()
      },
      GateState::Lowering => {
        "lower".to_string()
      },
      _ => {
        "nop".to_string()
      }
    }
  }
  fn verbose_input(&self, inp: &GateSensors) -> String {
    match self.state {
      GateState::Waiting => {
        format!("('bottom',{},{})",inp.car_at_gate,inp.car_just_existed)
      },
      GateState::Raising => {
        if inp.position == GatePosition::Top {
          format!("('top',{},{})",inp.car_at_gate,inp.car_just_existed)
        } else {
          format!("('middle',{},{})",inp.car_at_gate,inp.car_just_existed)
        }
      },
      GateState::Raised => {
        format!("('top',{},{})",inp.car_at_gate,inp.car_just_existed)
      },
      GateState::Lowering => {
        if inp.position == GatePosition::Bottom {
          format!("('bottom',{},{})",inp.car_at_gate,inp.car_just_existed)
        } else {
          format!("('middle',{},{})",inp.car_at_gate,inp.car_just_existed)
        }
      },
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::*;
  #[test]
  fn it_gets_next_values_gate_down_no_car() {
    let test = SimpleParkingGate::new(GateState::Waiting);
    // GatePosition::Bottom
    assert_eq!(
      test.get_next_values(
        GateState::Waiting,
        GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        }
      ),Ok((GateState::Waiting,"nop".to_string()))
    );
    assert_eq!(
      test.get_next_values(
        GateState::Raising,
        GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        }
      ),Ok((GateState::Raising,"raise".to_string()))
    );
    assert_eq!(
      test.get_next_values(
        GateState::Raised,
        GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        }
      ),Err("GatePosition and GateState sensors have invalid data".to_string())
    );
    assert_eq!(
      test.get_next_values(
        GateState::Lowering,
        GateSensors {
          car_at_gate: false,
          car_just_existed: false,
          position: GatePosition::Bottom
        }
      ),Ok((GateState::Waiting,"nop".to_string()))
    );
  }
}
