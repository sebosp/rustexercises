use std::io;
use std::fmt;
use std::ops::Add;
#[derive(Debug, PartialEq)]
pub struct Polynomial {
  items: Vec<f64>,
}
impl Polynomial {
  pub fn new(items: Vec<f64>) -> Self {
    Polynomial {
      items: items
    }
  }
  pub fn from_string(input: String) -> Self {
    Polynomial {
      items: input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .iter()
        .map(|x|
          x.parse::<f64>().unwrap()
        )
        .collect()
    }
  }
  pub fn add_polynomial(&self, addend: &Polynomial) -> Self {
    let mut res: Vec<f64> = Vec::new();
    // Initialize result to the items in first array
    for item in self.items.iter() {
        res.push(*item);
    }
    // Initialize the rest of the items
    for _ in res.len() - 1 .. addend.items.len() {
      res.insert(0,0f64);
    }
    let vec_len = addend.items.len();
    for (i, item) in addend.items.iter().rev().enumerate() {
        let mut sum = *item;
        if let Some(x) = res.get(vec_len - i) {
          sum += *x;
        }
        res[vec_len - i]=sum;
    }
    Polynomial {
      items: res
    }
  }
  pub fn to_string(&self) -> String {
    let mut output = String::new();
    let vec_len = self.items.len() - 1;
    for (i, item) in self.items.iter().enumerate() {
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
  pub fn solve(&self, _: i64) -> f64 {
      0f64
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
    assert_eq!(vec![1f64,2f64,3f64],testfrom.items);
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
  }
  #[test]
  fn test_solve() {
    let p1 = Polynomial::from_string("1 2 3".to_string());
    let p2 = Polynomial::from_string("100 200".to_string());
    assert_eq!(p1.solve(1),6.0);
    assert_eq!(p1.solve(-1),2.0);
    let p3 = &p1 + &p2;
    assert_eq!(p3.solve(10),1323.0);
  }
/*
>>> p1(1)
6.0
>>> p1(-1)
2.0
>>> (p1 + p2)(10)
1323.0
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
