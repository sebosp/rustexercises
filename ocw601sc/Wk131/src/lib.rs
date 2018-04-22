#![feature(test)]
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
pub enum FibonacciSolutions {
    Sequential(u8),
    RecursiveSlow(u8),
    Poles(u8),
    Phi(u8),
}
pub struct Fibonacci {
    pub target: FibonacciSolutions,
}
impl Fibonacci {
  // O(2^n)
  fn recurse_slow(target: u8) -> u64{
    match target {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => (Fibonacci::recurse_slow(target - 1) + Fibonacci::recurse_slow(target - 2))
    }
  }
  // O(n+2)
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
  // O(1)
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
  // O(1)
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
  pub fn solve(&self) -> u64{
      match self.target {
          FibonacciSolutions::Sequential(val) => Fibonacci::sequential(val),
          FibonacciSolutions::RecursiveSlow(val) => Fibonacci::recurse_slow(val),
          FibonacciSolutions::Poles(val) => Fibonacci::poles(val),
          FibonacciSolutions::Phi(val) => Fibonacci::phi(val)
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
        let solutions: Vec<u64> = vec![0,0,1,1,2,3,5,8,13,21,34,55,89,144];
        for i in 1u8..13u8 {
          println!("Testing {}",i);
          let ith = usize::from(i);
          let sequential = Fibonacci {
            target: FibonacciSolutions::Sequential(i)
          };
          assert_eq!(sequential.solve(),solutions[ith]);
          let recurse_slow = Fibonacci {
            target: FibonacciSolutions::RecursiveSlow(i)
          };
          assert_eq!(recurse_slow.solve(),solutions[ith]);
          let phi = Fibonacci {
            target: FibonacciSolutions::Phi(i)
          };
          assert_eq!(phi.solve(),solutions[ith]);
          let poles = Fibonacci {
            target: FibonacciSolutions::Poles(i)
          };
          assert_eq!(poles.solve(),solutions[ith]);
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
