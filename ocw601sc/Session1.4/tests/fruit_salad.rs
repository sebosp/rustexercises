extern crate additionals;
use additionals::*;
#[cfg(test)]
#[test]
fn test_fruit_salad() {
  let mut f1 = FruitSalad::new(vec!["melons".to_string(),"pineapples".to_string()],4);
  assert_eq!("4 servings of fruit salad with [\"melons\", \"pineapples\"]",f1.to_string());
  f1.add("mango".to_string());
  assert_eq!(f1.serve(),"enjoy".to_string());
  assert_eq!(f1.serve(),"enjoy".to_string());
  assert_eq!(f1.serve(),"enjoy".to_string());
  assert_eq!(f1.serve(),"enjoy".to_string());
  assert_eq!(f1.serve(),"sorry".to_string());
}
