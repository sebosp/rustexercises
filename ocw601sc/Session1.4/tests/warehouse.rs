extern crate additionals;
use additionals::*;
#[cfg(test)]
#[test]
fn test_waherouse_process() {
  let mut warehouse1 = Warehouse::new();
  let mut w1_transactions:Vec<TransactionOperation> = vec![];
  w1_transactions.push(TransactionOperation::new("ship".to_string(),"a".to_string(),1));
  warehouse1.process(&mut w1_transactions);
  assert_eq!(warehouse1.lookup("a".to_string()),0u32);
  w1_transactions.push(TransactionOperation::new("receive".to_string(),"b".to_string(),10));
  warehouse1.process(&mut w1_transactions);
  assert_eq!(warehouse1.lookup("b".to_string()),10u32);
  w1_transactions.push(TransactionOperation::new("receive".to_string(),"c".to_string(),10));
  w1_transactions.push(TransactionOperation::new("ship".to_string(),"c".to_string(),10));
  w1_transactions.push(TransactionOperation::new("ship".to_string(),"c".to_string(),10));
  warehouse1.process(&mut w1_transactions);
  assert_eq!(warehouse1.lookup("c".to_string()),0u32);
}
