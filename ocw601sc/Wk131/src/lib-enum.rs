#![feature(test)]
extern crate test;
use std::io;
// Note: 9223372036854775807 max u64
const PHI:f64 = 1.618033988749895; // from find_phi
pub fn read_u8() -> u8 {
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .expect("Failed to read line");
    input.trim()
        .to_string()
        .parse::<u8>()
        .expect("Failed to read u8")
}
enum Fibonacci {
    Sequential(u8),
    Recursive_Slow(u8),
    Poles(u8),
    Phi(u8),
}
impl Fibonacci {
  // O(2^n)
  pub fn fibo_recursive(self) -> u64{
      match self {
          Fibonacci::Sequential(0) => 0,
          Fibonacci::Sequential(1) => 0,
          Fibonacci::Sequential(2) => 1,
          Fibonacci::Sequential(x) => Fibonacci::Sequential(x - 1).calc() + Fibonacci::Sequential(x - 2).calc()
      }
  }
  // O(n+2)
  pub fn fibo_sequential(self) -> u64{
      let mut cache:Vec<u64> = vec![0u64,1u64];
      if self < 1 {
          return cache[0];
      }
      while self > 2 {
          let temp = cache[0];
          cache[0] = cache[1];
          cache[1] = cache[0] + temp;
          self = self - 1;
      }
      cache[1]
  }
  // O(1)
  pub fn fibo_phi(self) -> u64 {
      let target = i32::from(self);
      match target {
          0 => 0,
          1 => 0,
          2 => 1,
          _ => ((
                 PHI.powi(target-1) - (1.0 - PHI).powi(target-1)
                )/f64::from(5).sqrt()
               ).round() as u64
      }
  }
  // O(1)
  pub fn fibo_poles(self) -> u64 {
      let target = i32::from(self);
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
  fn calc(&self) -> u64{
      match self {
          Fibonacci::Sequential(val) => self.fibo_sequential(val),
          Fibonacci::Recursive(val) => self.fibo_recursive(val),
          Fibonacci::Poles(val) => self.fibo_poles(val),
          Fibonacci::Phi(val) => self.fibo_phi(val)
      }
  }
  pub fn print(self) {
    println!("---");
    for i in 1 .. self {
        println!("{},{}",i,self.calc(i));
    }
    println!("---");
  }
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
        assert_eq!(fibo_sequential(0),0);
        assert_eq!(fibo_sequential(2),1);
        assert_eq!(fibo_sequential(13),144);
        assert_eq!(fibo_recursive(0),0);
        assert_eq!(fibo_recursive(2),1);
        assert_eq!(fibo_recursive(13),144);
        assert_eq!(fibo_phi(0),0);
        assert_eq!(fibo_phi(2),1);
        assert_eq!(fibo_phi(13),144);
        assert_eq!(fibo_poles(0),0);
        assert_eq!(fibo_poles(2),1);
        assert_eq!(fibo_poles(13),144);
    }
    #[bench]
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
    }
}
