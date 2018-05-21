//! Polynomial is a collection of utils for solving Polynomials
extern crate fma;
extern crate rgsl;

use std::io;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;
use fma::*;
use rgsl::polynomials::quadratic_equations::*;
use rgsl::polynomials::cubic_equations::*;
#[derive(Debug, PartialEq)]
pub struct Polynomial {
  pub coeffs: Vec<f64>,
}
impl Polynomial {
  pub fn new(coeffs: Vec<f64>) -> Self {
    Polynomial {
      coeffs: coeffs
    }
  }
  /// Returns the coefficient at n position
  pub fn coeff(&self, input: usize) -> f64 {
      self.coeffs[self.coeffs.len() - input - 1]
  }
  /// Adds two polynomials
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
  /// Multiplies two polynomials
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
  /// Solves the Polynomial with Horners rule for x = val
  pub fn horner(&self, val: f64) -> f64 {
    let mut res = 0f64;
    for item in self.coeffs.iter(){
      res = res * val + *item;
    }
    res
  }
  /// Solves the Polynomial with Horners rule for x = val, additionally uses FMA (Fused
  /// Multiply-Add)
  pub fn horner_fma(&self, val: f64) -> f64 {
    let mut res = 0f64;
    for item in self.coeffs.iter(){
      res = fma(res,val,*item);
    }
    res
  }
  /// Sequentially solves the Polynomial
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
  /// From a space separated String creates a Polynomial
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
  /// Converts a Polynomial to a String
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
  pub fn roots(&self) -> Result<Vec<f64> , &'static str>{
    match self.coeffs.len() {
      1|2|3 => {
        let mut vals: (f64,f64,f64) = (0.,0.,0.);
        if self.coeffs.len() > 2 {
          vals.2 = self.coeffs[2];
        }
        if self.coeffs.len() > 1 {
          vals.1 = self.coeffs[1];
        }
        vals.0 = self.coeffs[0];
        let mut x0:f64 = 1.;
        let mut x1:f64 = 1.;
        let res = poly_solve_quadratic(
            vals.0,
            vals.1,
            vals.2,
            &mut x0,
            &mut x1
            );
        println!("quadratic res = {} ({},{})",res,x0,x1);
        Ok(vec![x0,x1])
      },
      4 => {
        if self.coeffs[0] != 1. {
          Err("Order too high to solve for roots.")
        } else {
          let mut vals: (f64,f64,f64) = (0.,0.,0.);
          if self.coeffs.len() > 2 {
            vals.2 = self.coeffs[2];
          }
          if self.coeffs.len() > 1 {
            vals.1 = self.coeffs[1];
          }
          vals.0 = self.coeffs[0];
          let mut x0:f64 = 0.;
          let mut x1:f64 = 0.;
          let mut x2:f64 = 0.;
          let res = poly_solve_cubic(
              vals.0,
              vals.1,
              vals.2,
              &mut x0,
              &mut x1,
              &mut x2
              );
          println!("cubic res = {} ({},{},{})",res,x0,x1,x2);
          Ok(vec![x0,x1,x2])
        }
      },
      _ => Err("Order too high to solve for roots.")
    }
  }
}
/// Implements the Display Trait for Polynomial
impl fmt::Display for Polynomial {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    /// Expected output 1.000 z**2 + 2.000 z + 3.000
    write!(f, "{}", self.to_string())
  }
}
/// Implements the Add Trait for Polynomial with separate Lifetimes.
impl<'r, 'a> Add<&'a Polynomial> for &'r Polynomial {
    type Output = Polynomial;
    fn add(self, other: &Polynomial) -> Polynomial {
        self.add_polynomial(other)
    }
}
/// Implements the Multiply Trait for Polynomial with separate Lifetimes.
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
