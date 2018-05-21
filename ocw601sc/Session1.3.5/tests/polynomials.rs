#![feature(test)]
extern crate polynomial;
use polynomial::*;
extern crate test;
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
  let p4 = Polynomial::from_string("2 -6 2 -1".to_string());
  assert_eq!(p4.solve(3f64),5.0);
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
#[test]
fn test_horner() {
  let p1 = Polynomial::from_string("1 2 3".to_string());
  let p2 = Polynomial::from_string("100 200".to_string());
  assert_eq!(p1.horner(1f64),6.0);
  assert_eq!(p1.horner(-1f64),2.0);
  let p3 = &p1 + &p2;
  assert_eq!(p3.horner(10f64),1323.0);
  let p4 = Polynomial::from_string("2 -6 2 -1".to_string());
  assert_eq!(p4.horner(3f64),5.0);
}
#[bench]
fn bench_horner(b: &mut test::Bencher) {
    b.iter(|| {
      Polynomial::from_string("2 -6 2 -1".to_string()).horner(3f64);
    })
}
#[bench]
fn bench_horner_fma(b: &mut test::Bencher) {
    b.iter(|| {
      Polynomial::from_string("2 -6 2 -1".to_string()).horner_fma(3f64);
    })
}
#[bench]
fn bench_solve(b: &mut test::Bencher) {
    b.iter(|| {
      Polynomial::from_string("2 -6 2 -1".to_string()).solve(3f64);
    })
}
#[test]
fn test_roots() {
  let p0 = Polynomial::from_string("0 3 -21".to_string());
  assert_eq!(p0.roots(), Ok(vec![7.,1.]));
  let p1 = Polynomial::from_string("1 2 3".to_string());
  assert_eq!(p1.roots(), Ok(vec![1.,1.]));
  let p2 = Polynomial::from_string("100 200".to_string());
  assert_eq!(p2.roots(), Ok(vec![-2.,0.]));
  let p3 = Polynomial::from_string("3 2 -1".to_string());
  assert_eq!(p3.roots(), Ok(vec![-1., 0.33333333333333331]));
  let p4 = Polynomial::from_string("4 -20 26".to_string());
  assert_eq!(p4.roots(), Ok(vec![1.0,1.0]));
}
