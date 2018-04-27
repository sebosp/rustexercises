#![feature(test)]
//! `fibo` is a collection of solutions for the Fibonacci sequence
extern crate test;
use std::io;
// Note: 9223372036854775807 max u64
pub fn read_u8() -> u8 {
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .expect("Failed to read line");
    input.trim()
        .to_string()
        .parse::<u8>()
        .expect("Failed to read u8")
}
/// Different solutions for the Fibonacci sequence
pub struct FibonacciRecursiveSlow {
    pub target: u8,
}
pub struct FibonacciPoles {
    pub target: u8,
}
pub struct FibonacciPhi {
    pub target: u8,
}
pub struct FibonacciRecursiveCache {
    pub target: u8,
    cache: Vec<u64>,
}
pub struct FibonacciSequential {
    pub target: u8,
    cache: Vec<u64>,
}
pub trait Solvable {
    fn solve(&mut self) -> u64;
}
/// Implementation of the solutions without cache
/// For convenience, F(0) is assumed to return 0, altho this might not be true.
impl FibonacciRecursiveSlow {
  /// Requests recursively cost is O(2^n)
  fn rec_solve(&self, target: u8) -> u64{
    match target {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => (self.rec_solve(target - 1) + self.rec_solve(target - 2))
    }
  }
}
impl Solvable for FibonacciRecursiveSlow {
  fn solve(&mut self) -> u64 {
      self.rec_solve(self.target)
  }
}
impl Solvable for FibonacciPhi {
  /// Uses the golden ratio to calculate the result.
  /// Cost: O(1)
  fn solve(&mut self) -> u64 {
    let golden_ratio:f64 = 1.618033988749895; // from find_phi
    let target = i32::from(self.target);
    match target {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => ((
               golden_ratio.powi(target-1) - (1.0 - golden_ratio).powi(target-1)
              )/f64::from(5).sqrt()
             ).round() as u64
    }
  }
}
impl Solvable for FibonacciPoles {
  /// Uses poles to calculate the result.
  /// Cost:  O(1)
  fn solve(&mut self) -> u64 {
    let target = i32::from(self.target);
    match target {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => ( 
              ( ((1.0 + f64::from(5).sqrt()) / 2.0).powi(target-1)
               +((1.0 - f64::from(5).sqrt()) / 2.0).powi(target-1)
              )/f64::from(5).sqrt()).round() as u64
    }
  }
}
/// Implementation of the solutions without cache
/// For convenience, F(0) is assumed to return 0, altho this might not be true.
impl Solvable for FibonacciRecursiveCache {
  /// Requests recursively the solutions and caches
  /// results, the cost is O(n)
  fn solve(&mut self) -> u64 {
      let target = usize::from(self.target);
      self.rec_solve(target)
  }
}
impl FibonacciPhi {
  fn new(target: u8) -> Self {
    FibonacciPhi {
        target: target,
    }
  }
}
impl FibonacciPoles {
  fn new(target: u8) -> Self {
    FibonacciPoles {
        target: target,
    }
  }
}
impl FibonacciRecursiveSlow {
  fn new(target: u8) -> Self {
    FibonacciRecursiveSlow {
        target: target,
    }
  }
}
impl FibonacciRecursiveCache {
  fn rec_solve(&mut self, target: usize) -> u64{
    if let Some(_) = self.cache.get(target as usize){
        return self.cache[target]
    }
    self.cache[target] = self.rec_solve(target - 2) + self.rec_solve(target - 1);
    self.cache[target]
  }
  fn new(target: u8) -> Self {
    FibonacciRecursiveCache {
        cache: vec![0u64,0u64,1u64],
        target: target,
    }
  }
}
impl FibonacciSequential {
  fn new(target: u8) -> Self {
    FibonacciSequential {
        cache: vec![0u64,1u64],
        target: target,
    }
  }
}
impl Solvable for FibonacciSequential {
  /// Uses a two items vector with the current solutions
  /// Cost: O(n+2)? There's always need to overwrite the values.
  fn solve(&mut self) -> u64{
    if self.target < 2 {
        return self.cache[0]
    }
    while self.target > 2 {
        let temp = self.cache[0];
        self.cache[0] = self.cache[1];
        self.cache[1] = self.cache[0] + temp;
        self.target = self.target - 1;
    }
    self.cache[1]
  }
}
pub struct Fibonacci {
    target: u8,
    target_solution: String,
}
impl Fibonacci {
  /// Returns a new instance of Fibonacci.
  pub fn new(solution: &String, target: u8) -> Self {
      Fibonacci {
          target: target,
          target_solution: solution.clone()
      }
  }
  pub fn solve(self) -> u64 {
      match self.target_solution.as_ref() {
          "poles" =>
              FibonacciPoles::new(self.target).solve(),
          "recursive_slow"|"rec_slow" =>
              FibonacciRecursiveSlow::new(self.target).solve(),
          "phi" =>
              FibonacciPhi::new(self.target).solve(),
          "recursive_cache"|"rec_cache" =>
              FibonacciRecursiveCache::new(self.target).solve(),
          _ =>
              FibonacciSequential::new(self.target).solve(),
      }
  }
/*  fn Display(self) {
    println!("---");
    for i in 1 .. self.target {
      match self.target {
          FibonacciSolutions::Sequential(val) => Fibonacci::sequential(i),
          FibonacciSolutions::RecursiveSlow(val) => Fibonacci::recurse_slow(i),
          FibonacciSolutions::Poles(val) => Fibonacci::poles(i),
          FibonacciSolutions::Phi(val) => Fibonacci::phi(i)
      }
    }
    println!("---");
  }*/
}
/// Helper used once to calculate the golden ratio.
/// The highest the target, the more precise it will be, but we're not checking
/// for overflows on u64 either...
pub fn find_phi(mut target: u8) -> f64{
    let mut cache = vec![0u64,1u64];
    while target > 1 {
        let temp = cache[0];
        cache[0] = cache[1];
        cache[1] = cache[0] + temp;
        target = target - 1;
    }
    cache[1] as f64/cache[0] as f64

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        // Known solutions
        let solutions: Vec<u64> = vec![0,0,1,1,2,3,5,8,13,21,34,55,89,144];
        let solution_types: Vec<String> = vec![
            "seq".to_string(),
            "recurse_slow".to_string(),
            "phi".to_string(),
            "poles".to_string(),
            "recurse_cache".to_string()
        ];
        for solution_type in solution_types {
            for i in 1u8..13u8 {
                let ith = usize::from(i);
                assert_eq!(Fibonacci::new(&solution_type,i).solve(),solutions[ith]);
            }
        }
    }
/*    #[bench]
    fn bench_fibo_sequential(b: &mut test::Bencher) {
        b.iter(|| {
            fibo_sequential(55)
        })
    }
    #[bench]
    fn bench_fibo_phi(b: &mut test::Bencher) {
        b.iter(|| {
            fibo_phi(55)
        })
    }
    #[bench]
    fn bench_fibo_poles(b: &mut test::Bencher) {
        b.iter(|| {
            fibo_poles(55)
        })
    }
    #[bench]
    fn bench_fibo_recursive(b: &mut test::Bencher) {
        b.iter(|| {
            fibo_recursive(55)
        })
    }*/
}
