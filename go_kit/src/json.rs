use super::*;
/// The JSON type: implements `FromData` and `Responder`, allowing you to easily
/// consume and respond with JSON.

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_gets_json() {
      let test = StringService::new("test".to_owned());
      let test_json_string = serde_json::to_string(&test).unwrap();
      assert_eq!(
        test_json_string,
        json!({"data":"test"}).to_string()
      );
  }
}

