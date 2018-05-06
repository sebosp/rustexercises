use std::io;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;
#[derive(Debug, PartialEq)]
pub struct Polynomial {
  coeffs: Vec<f64>,
}
impl Polynomial {
  pub fn new(coeffs: Vec<f64>) -> Self {
    Polynomial {
      coeffs: coeffs
    }
  }
  pub fn coeff(&self, input: usize) -> f64 {
      self.coeffs[self.coeffs.len() - input - 1]
  }
  pub fn add_polynomial(&self, addend: &Polynomial) -> Self {
    let mut res: Vec<f64> = Vec::new();
    // Initialize result to the coeffs in first array
    for item in self.coeffs.iter() {
        res.push(*item);
    }
    // Initialize the rest of the coeffs
    for _ in res.len() .. addend.coeffs.len() {
      res.insert(0,0f64);
    }
    let vec_len = res.len();
    for (i, item) in addend.coeffs.iter().rev().enumerate() {
        let mut sum = *item;
        if let Some(x) = res.get(vec_len - i - 1) {
          sum += *x;
        }
        res[vec_len - i - 1]=sum;
    }
    Polynomial {
      coeffs: res
    }
  }
  pub fn mul_polynomial(&self, multiplicand: &Polynomial) -> Self {
    let mut res: Vec<f64> = Vec::new();
    // Initialize the output array
    for _ in 0 .. (self.coeffs.len() + multiplicand.coeffs.len() - 1) {
      res.insert(0,0f64);
    }
    for (i, item_i) in self.coeffs.iter().enumerate() {
      for (j, item_j) in multiplicand.coeffs.iter().enumerate() {
        res[i+j] += *item_i * *item_j;
      }
    }
    Polynomial {
      coeffs: res
    }
  }
  pub fn solve(&self, val: f64) -> f64 {
    let mut res = 0f64;
    for (i, item) in self.coeffs.iter().rev().enumerate() {
      let exponential:i32 = i32::from(i as u8);
      if exponential > 1 {
          res += *item * val.powi(exponential);
      }else if exponential == 1 {
          res += *item * val;
      }else{
          res += *item;
      }
    }
    res
  }
  pub fn from_string(input: String) -> Self {
    Polynomial {
      coeffs: input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .iter()
        .map(|x|
          x.parse::<f64>().unwrap()
        )
        .collect()
    }
  }
  pub fn to_string(&self) -> String {
    let mut output = String::new();
    let vec_len = self.coeffs.len() - 1;
    for (i, item) in self.coeffs.iter().enumerate() {
      if *item == 0f64 {
        continue;
      }
      let exponential = vec_len - i;
      if output.len() == 0 {
        if *item < 0f64 {
          output.push_str(&"-".to_string());
        }
      } else {
        if *item >= 0f64 {
          output.push_str(&" + ".to_string());
        } else {
          output.push_str(&" - ".to_string());
        }
      }
      output.push_str(&format!("{:.*}",3,item.abs()));
      if exponential > 1 {
        output.push_str(&" z**".to_string());
        output.push_str(&exponential.to_string());
      }else if exponential == 1 {
        output.push_str(&" z".to_string());
      }
    }
    output
  }
}
impl fmt::Display for Polynomial {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    /// Expected output 1.000 z**2 + 2.000 z + 3.000
    write!(f, "{}", self.to_string())
  }
}
impl<'r, 'a> Add<&'a Polynomial> for &'r Polynomial {
    type Output = Polynomial;
    fn add(self, other: &Polynomial) -> Polynomial {
        self.add_polynomial(other)
    }
}
impl<'r, 'a> Mul<&'a Polynomial> for &'r Polynomial {
    type Output = Polynomial;
    fn mul(self, other: &Polynomial) -> Polynomial {
        self.mul_polynomial(other)
    }
}
/// Helper functions
pub fn read_line() -> String {
  let mut input = String::new();
  io::stdin().read_line(&mut input)
    .expect("Failed to read line");
  input
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_from_string() {
    let testfrom = Polynomial::from_string("1 2 3".to_string());
    assert_eq!(vec![1f64,2f64,3f64],testfrom.coeffs);
  }
  #[test]
  fn test_to_string() {
    assert_eq!(Polynomial::from_string("8".to_string()).to_string(),"8.000".to_string());
    assert_eq!(Polynomial::from_string("3 0 0 0".to_string()).to_string(),"3.000 z**3".to_string());
    assert_eq!(Polynomial::from_string("5 6 7".to_string()).to_string(),"5.000 z**2 + 6.000 z + 7.000".to_string());
    assert_eq!(Polynomial::from_string("-5 -6 7".to_string()).to_string(),"-5.000 z**2 - 6.000 z + 7.000".to_string());
  }
  #[test]
  fn test_add() {
    let p1 = Polynomial::from_string("1 2 3".to_string());
    // 1.000 z**2 + 2.000 z + 3.000
    let p2 = Polynomial::from_string("100 200".to_string());
    // 100.000 z + 200.000
    let p3 = &p1 + &p2;
    assert_eq!(p3.to_string(),"1.000 z**2 + 102.000 z + 203.000".to_string());
    let p4 = &p2 + &p1;
    assert_eq!(p4.to_string(),"1.000 z**2 + 102.000 z + 203.000".to_string());
    let p5 = Polynomial::from_string("1 102 203".to_string());
    assert_eq!(p4,p5);
  }
  #[test]
  fn test_coeff() {
    let p1 = Polynomial::from_string("1 -7 10 -4 6".to_string());
    assert_eq!(p1.coeff(3),-7f64);
  }
  #[test]
  fn test_solve() {
    let p1 = Polynomial::from_string("1 2 3".to_string());
    let p2 = Polynomial::from_string("100 200".to_string());
    assert_eq!(p1.solve(1f64),6.0);
    assert_eq!(p1.solve(-1f64),2.0);
    let p3 = &p1 + &p2;
    assert_eq!(p3.solve(10f64),1323.0);
    assert_eq!((&p1 + &p2).solve(10f64),1323.0);
  }
  #[test]
  fn test_multiply() {
    let p1 = Polynomial::from_string("1 2 3".to_string());
    assert_eq!((&p1 * &p1).to_string(),"1.000 z**4 + 4.000 z**3 + 10.000 z**2 + 12.000 z + 9.000".to_string());
    let p2 = Polynomial::from_string("100 200".to_string());
    assert_eq!((&(&p1 * &p2) + &p1).to_string(),"100.000 z**3 + 401.000 z**2 + 702.000 z + 603.000".to_string());
    let p3 = Polynomial::from_string("4 -5".to_string());
    let p4 = Polynomial::from_string("2 3 -6".to_string());
    assert_eq!(&p3 * &p4,Polynomial::from_string("8 2 -39 30".to_string()));
  }
/*
>>> p1.mul(p1)
1.000 z**4 + 4.000 z**3 + 10.000 z**2 + 12.000 z + 9.000
>>> p1 * p1
1.000 z**4 + 4.000 z**3 + 10.000 z**2 + 12.000 z + 9.000
>>> p1 * p2 + p1
100.000 z**3 + 401.000 z**2 + 702.000 z + 603.000
>>> p1.roots()
[(-1+1.4142135623730947j), (-1-1.4142135623730947j)]
>>> p2.roots()
[-2.0]
>>> p3 = Polynomial([3, 2, -1])
>>> p3.roots()
[-1.0, 0.33333333333333331]
>>> (p1 * p1).roots()
Order too high to solve for roots.*/
}
