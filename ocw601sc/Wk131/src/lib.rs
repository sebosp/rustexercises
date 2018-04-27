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
pub enum FibonacciSolutions {
    Sequential(u8),
    RecursiveSlow(u8),
    RecursiveCache(u8),
    Poles(u8),
    Phi(u8),
}
/// A struct with a target Fibonacci Solution enum.
pub struct Fibonacci {
    pub target: FibonacciSolutions,
    cache: Vec<u64>,
}
/// Implementation of the solutions.
/// For convenience, F(0) is assumed to return 0, altho this might not be true.
impl Fibonacci {
  /// Requests recursively the solutions but does not cache
  /// results, meaning the cost is O(2^n)
  fn recurse_slow(target: u8) -> u64{
    match target {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => (Fibonacci::recurse_slow(target - 1) + Fibonacci::recurse_slow(target - 2))
    }
  }
  /// Requests recursively the solutions and caches
  /// results, the cost is O(n)
  fn recurse_cache_run(&mut self, target: usize) -> u64{
    if let Some(_) = self.cache.get(target as usize){
        return self.cache[target]
    }
    self.cache[target] = Fibonacci::recurse_cache_run(self, target - 2) + Fibonacci::recurse_cache_run(self, target - 1);
    self.cache[target]
  }
  fn recurse_cache(&mut self, target: u8) -> u64{
      self.cache = vec![0u64,0u64,1u64];
      Fibonacci::recurse_cache_run(self, target as usize)
  }
 
  /// Uses a two items vector with the current solutions
  /// Cost: O(n+2)? There's always need to overwrite the values.
  fn sequential(mut target: u8) -> u64{
    let mut cache:Vec<u64> = vec![0u64,1u64];
    if target < 2 {
        return cache[0]
    }
    while target > 2 {
        let temp = cache[0];
        cache[0] = cache[1];
        cache[1] = cache[0] + temp;
        target = target - 1;
    }
    cache[1]
  }
  /// Uses the golden ratio to calculate the result.
  /// Cost: O(1)
  fn phi(target: u8) -> u64 {
    let golden_ratio:f64 = 1.618033988749895; // from find_phi
    let target = i32::from(target);
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
  /// Uses poles to calculate the result.
  /// Cost:  O(1)
  fn poles(target: u8) -> u64 {
    let target = i32::from(target);
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
  /// Depending on the target enum type, calculates on a solution.
  pub fn solve(&mut self) -> u64{
      match self.target {
          FibonacciSolutions::Sequential(val) => Fibonacci::sequential(val),
          FibonacciSolutions::RecursiveSlow(val) => Fibonacci::recurse_slow(val),
          FibonacciSolutions::Poles(val) => Fibonacci::poles(val),
          FibonacciSolutions::Phi(val) => Fibonacci::phi(val),
          FibonacciSolutions::RecursiveCache(val) => Fibonacci::recurse_cache(self,val)
      }
  }
  /// Returns a new instance of Fibonacci.
  pub fn new(target_solution: &String, target: u8) -> Fibonacci{
      match target_solution.as_ref() {
          "poles" =>
              Fibonacci {
                  target: FibonacciSolutions::Poles(target),
                  cache: Vec::new()
              },
          "recursive_slow"|"rec_slow" =>
              Fibonacci {
                  target: FibonacciSolutions::RecursiveSlow(target),
                  cache: Vec::new()
              },
          "phi" =>
              Fibonacci {
                  target: FibonacciSolutions::Phi(target),
                  cache: Vec::new()
              },
          "recursive_cache"|"rec_cache" =>
              Fibonacci {
                  target: FibonacciSolutions::RecursiveCache(target),
                  cache: Vec::new()
              },
          _ =>
              Fibonacci {
                  target: FibonacciSolutions::Sequential(target),
                  cache: Vec::new()
              }
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
