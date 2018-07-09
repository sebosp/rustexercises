pub mod feedback;
pub mod feedback2;
pub trait StateMachine {
  type StateType;
  type InputType;
  type OutputType;
  fn get_next_values(&self, state: &Self::StateType, inp: &Self::InputType) -> Result<(Self::StateType,Option<Self::OutputType>),String>;
}
