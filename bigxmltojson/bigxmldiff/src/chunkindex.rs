//! `ChunkIndex`
//! Provides an indexed checksum collection for chunks of data

extern crate crypto;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::collections::BTreeMap;

pub struct Chunk {
  pub chunk_id: String,
  pub checksum: String,
  pub offset: usize,
}

pub struct ChunkIndex {
  file: String,
  chunks: BTreeMap<String,String>,
}
impl Chunk {
  pub fn new(chunk_id: String, checksum: String, offset: usize) -> Chunk {
    Chunk{
      chunk_id: chunk_id,
      checksum: checksum,
      offset: offset,
    }
  }
}
impl ChunkIndex {
  pub fn new(file: &String) -> ChunkIndex {
    ChunkIndex{
      file: file.clone(),
      chunks: BTreeMap::new(),
    }
  }
  pub fn insert(&mut self, chunk_id: String, payload: String) -> bool {
    if self.search(&chunk_id).is_none() {
      self.chunks.insert(chunk_id, payload);
      true
    } else {
      false
    }
  }
  pub fn display(&self) {
    println!("Displaying chunk indexes for file {}", &self.file);
    for (chunk_id, payload) in &self.chunks{
      println!("{}: \"{}\"", chunk_id, payload);
    }
  }
  pub fn search(&self, chunk_id: &String) -> Option<&String> {
    self.chunks.get(chunk_id)
  }
}
/// `calculate_checksum` gets a data chunk and creates a SHA256 out of it.
pub fn calculate_checksum(input: &String) -> String {
  let mut sha = Sha256::new();
  // Sort the lines, in case the items/lines change position.
  let mut lines = input.lines().map(|s| s.to_owned()).collect::<Vec<String>>();
  lines.sort_unstable();
  sha.input_str(&lines.join("\n"));
  sha.result_str()
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_gets_sha256() {
    let test_data = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(calculate_checksum(&test_data),calculate_checksum(&test_data));
  }
}
