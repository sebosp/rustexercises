//! OCW601 Work 1.3.4 2D Vector utils
use std::io;
use std::fmt;
#[derive(Debug)]
pub struct V2 {
  x: f64,
  y: f64,
}
/// Setters and Getters probably not great for Rust.
impl V2 {
  /// A setter for x field
  pub fn get_x(&self) -> f64 {
    self.x
  }
  /// A setter for y field
  pub fn get_y(&self) -> f64 {
    self.y
  }
  pub fn new(x: f64, y: f64) -> Self {
    V2 { x,y }
  }
  pub fn add(&self, right: &V2) -> Self {
    V2 {
      x: self.get_x() + right.get_x(),
      y: self.get_y() + right.get_y()
    }
  }
  pub fn multiply(&self, multiplicand: f64) -> Self {
    V2 {
      x: self.get_x() * multiplicand,
      y: self.get_y() * multiplicand
    }
  }
}
impl fmt::Display for V2 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "V2[{},{}]\n", self.get_x(), self.get_y())
  }
}
pub fn read_f64() -> f64 {
  let mut input = String::new();
  io::stdin().read_line(&mut input)
    .expect("Failed to read line");
  input.trim()
    .to_string()
    .parse::<f64>()
    .expect("Failed to read f64")
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_getters() {
    let test_v2 = V2 {
      x: 1.0,
      y: 2.0,
    };
    assert_eq!(&test_v2.get_x(), &f64::from(1.0));
    assert_eq!(&test_v2.get_y(), &f64::from(2.0));
  }
  #[test]
  fn test_add() {
    let vec1 = V2 {
      x: 1.0,
      y: 2.0,
    };
    let vec2 = V2 {
      x: 1.0,
      y: 2.0,
    };
    let vec3 = vec1.add(&vec2);
    assert_eq!(&vec3.get_x(), &f64::from(2.0));
    assert_eq!(&vec3.get_y(), &f64::from(4.0));
  }
  #[test]
  fn test_multiply() {
    let vec1 = V2 {
      x: 1.0,
      y: 2.0,
    };
    let vec2 = vec1.multiply(2.0);
    assert_eq!(&vec2.get_x(), &f64::from(2.0));
    assert_eq!(&vec2.get_y(), &f64::from(4.0));
  }
}
