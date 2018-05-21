extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;
use std::io;
use std::collections::HashMap;
pub fn is_palindrome(input: &str) -> bool {
  let graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  graphemes.iter().take(graphemes.len() + 1 / 2 as usize).zip(graphemes.iter().rev().take(graphemes.len() + 1 / 2 as usize)).all(|(x,y)| x == y)
}
pub fn is_substring(input: &str, fragment: &str) -> bool {
  let input_graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  let fragment_graphemes = UnicodeSegmentation::graphemes(fragment, true).collect::<Vec<&str>>();
  let mut match_offset = 0;
  for (input_index,input_grapheme) in input_graphemes.iter().enumerate() {
    if fragment.len() - match_offset > input_graphemes.len() - input_index {
      return false
    }
    if input_grapheme == &fragment_graphemes[match_offset] {
      match_offset += 1;
    }
    if match_offset == fragment.len() {
      return true
    }
  }
  false
}
pub fn extract_tags(input: &str) -> Result<Vec<String>, String> {
  let mut res: Vec<String> = vec![];
  let mut current_tag: String = "".to_string();
  let input_graphemes = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();
  let mut open_tag: bool = false;
  for grapheme in input_graphemes.iter() {
    if grapheme == &"]".to_string() {
      if ! open_tag {
        return Err("Unmatched open tag".to_string())
      }
      if current_tag.len() > 0 {
        res.push(current_tag);
        current_tag = "".to_string();
      }
      open_tag = false;
      continue;
    }
    if open_tag {
      current_tag.push_str(grapheme);
    }
    if grapheme == &"[".to_string() {
      if open_tag {
        return Err("Tags within tags are not supported".to_string())
      } else {
        open_tag = true;
      }
    }
  }
  if open_tag {
    Err("Unmatched close tag".to_string())
  } else {
    Ok(res)
  }
}
pub struct FruitSalad {
  fruits: Vec<String>,
  servings: u32,
}
impl FruitSalad {
  pub fn new(fruits: Vec<String>, servings: u32) -> Self {
    FruitSalad {
      fruits: fruits,
      servings: servings,
    }
  }
  pub fn to_string(&self) -> String {
    format!("{} servings of fruit salad with {:?}",self.servings, self.fruits)
  }
  pub fn add(&mut self, new_fruit: String) {
    self.fruits.push(new_fruit);
  }
  pub fn serve(&mut self) -> String {
    if self.servings > 0 {
      self.servings = self.servings - 1;
      "enjoy".to_string()
    } else {
      "sorry".to_string()
    }
  }
}
enum TransactionType {
  Receive,
  Ship,
}
pub struct TransactionOperation {
  tx_type: TransactionType,
  item: String,
  quantity: u32,
}
impl TransactionOperation {
  pub fn new(transaction_type: String,item: String, quantity: u32) -> Self {
    let mut tx_type:TransactionType = TransactionType::Receive;
    if &transaction_type == &"ship".to_string() {
      tx_type = TransactionType::Ship;
    }
    TransactionOperation {
      tx_type: tx_type,
      item: item,
      quantity: quantity
    }
  }
}
pub struct Warehouse {
  items: HashMap<String,u32>,
}
impl Warehouse {
  pub fn process(&mut self, transactions: &mut Vec<TransactionOperation>) {
    for curr_transaction in transactions.drain(..) {
      let count = self.items.entry(curr_transaction.item.clone()).or_insert(0);
      match curr_transaction.tx_type {
        TransactionType::Receive => *count += curr_transaction.quantity,
        TransactionType::Ship => if *count < curr_transaction.quantity {
          *count = 0;
        }else{
          *count -= curr_transaction.quantity;
        },
      }
    }
  }
  pub fn lookup(&self,item: String) -> u32 {
    match self.items.get(&item) {
      Some(x) => *x,
      None => 0u32
    }
  }
  pub fn new() -> Self {
    Warehouse {
      items: HashMap::new(),
    }
  }
}
/// Helper functions
pub fn read_line() -> String {
  let mut input = String::new();
  io::stdin().read_line(&mut input)
    .expect("Failed to read line");
  input
}
