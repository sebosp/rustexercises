//! `ChunkIndex`
//! Provides an indexed checksum collection for chunks of data
//!

extern crate crypto;
extern crate std;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

pub struct Chunk {
  pub chunk_id: String,
  pub checksum: String,
  pub offset: usize,
}

pub struct ChunkIndex {
  pub file: String,
  pub chunks: BTreeMap<String,String>,
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
  pub fn remove(&mut self, chunk_id: &String) {
    self.chunks.remove(chunk_id);
  }
  pub fn store(&self, file: &String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    for (chunk_id, payload) in &self.chunks {
      file.write_all(format!("{}&{}\n", chunk_id, payload).as_bytes())?;
    }
    Ok(())
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

/// `calculate_diff` stores the difference of the two chunk_indexes in the output file.
/// The first file is seen as the "old version" and the new file as the "new version".
/// This calculation yields 3 files that are sorted by file byte offset to allow easy fseek.
/// After each of these operations, the entries are deleted from each ChunkIndex
/// - When an entry exists on both idx1 and idx2, and they are the same:
///   These records are ignored.
/// - When an entry exists on both idx1 and idx2, and they are different:
///   idx2 version will be recognized the new version.
///   These records are added to the ".modified" file
/// - When an entry exist only in idx1 and not in idx2:
///   The records are added to the ".deleted" file
/// - When an entry exist only in idx2:
///   The records are added to the ".added" file.
pub fn calculate_diff(idx1: &mut ChunkIndex, idx2: &mut ChunkIndex, verbosity: i8) -> (Vec<usize>,Vec<usize>,Vec<usize>) {
  // ChunkIndexes marked fol deletion
  let mut chunk1_todelete: Vec<String> = vec![];
  let mut chunk2_todelete: Vec<String> = vec![];
  // Vectors that store the offsets, in the spirit of git status:
  let mut added: Vec<usize> = vec![]; // Offset in second file, contains new val.
  let mut modified: Vec<usize> = vec![]; // Offset in second file, contains new val.
  let mut deleted: Vec<usize> = vec![]; // Offset in first file, contains removed val.
  // Placeholders
  let mut offset: usize;
  let mut chunk1_payload: Vec<String>;
  let mut chunk2_payload: Vec<String>;
  for (chunk1_id, chunk1_data) in &idx1.chunks {
    chunk1_payload = chunk1_data.split('&').map(|s| s.to_owned()).collect::<Vec<_>>();
    if chunk1_payload.len() != 2 {
      panic!("Chunk1 Index is corrupt. The payload does not contain 2 fields");
    }
    match idx2.chunks.get(chunk1_id) {
      Some(chunk2_data) => {
        // Check for equality, see if it has been updated.
        // The payload contains "checksum&offset"
        chunk2_payload = chunk2_data.split('&').map(|s| s.to_owned()).collect::<Vec<_>>();
        if chunk2_payload.len() != 2 {
          panic!("Chunk2 Index is corrupt. The payload does not contain 2 fields");
        }
        if chunk1_payload[0] != chunk2_payload[0] {
          if verbosity > 0 {
            println!("+ modified: {}: (prev: {}, new: {})",chunk1_id,chunk1_payload[0],chunk2_payload[0]);
          }
          offset = chunk2_payload[1].parse::<usize>().expect("Unable to parse chunk2 offset to usize");
          modified.push(offset);

        } else {
          if verbosity > 3 {
            println!(": unchanged: {}: ({})",chunk1_id,chunk1_payload[0]);
          }
        }
        chunk1_todelete.push(chunk1_id.to_owned());
        chunk2_todelete.push(chunk1_id.to_owned());
      },
      None => {
        if verbosity > 0 {
          println!("- deleted: {}: ({})",chunk1_id,chunk1_payload[0]);
        }
        offset = chunk1_payload[1].parse::<usize>().expect("Unable to parse chunk1 offset to usize");
        deleted.push(offset);
      }
    }
  }
  for todelete in chunk1_todelete {
    idx1.chunks.remove(&todelete);
  }
  for todelete in chunk2_todelete {
    idx2.chunks.remove(&todelete);
  }
  for (chunk2_id, chunk2_data) in &idx2.chunks {
    chunk2_payload = chunk2_data.split('&').map(|s| s.to_owned()).collect::<Vec<_>>();
    if chunk2_payload.len() != 2 {
      panic!("Chunk2 Index is corrupt. The payload does not contain 2 fields");
    }
    match idx1.chunks.get(chunk2_id) {
      Some(_) => println!("* this ID have been removed: {}: ({})",chunk2_id,chunk2_payload[0]),
      None => {
        offset = chunk2_payload[1].parse::<usize>().expect("Unable to parse chunk2 offset to usize");
        added.push(offset);
      }
    }
  }
  (added, modified, deleted)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_gets_sha256() {
    let test_data = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(calculate_checksum(&test_data),calculate_checksum(&test_data));
  }
  #[test]
  fn it_calculates_diff() {
    let verbosity = -1i8;
    let mut idx1 = ChunkIndex::new(&"NONE".to_owned());
    let mut idx2 = ChunkIndex::new(&"NONE".to_owned());
    // These records exist in both indexes.
    idx1.insert("sameid".to_owned(),"SHA1&0".to_owned());
    idx2.insert("sameid".to_owned(),"SHA1&0".to_owned());
    // This record exists only in the first index.
    idx1.insert("deletedid".to_owned(),"SHA1&1".to_owned());
    // These records have been modified.
    idx1.insert("changedid".to_owned(),"SHA1&2".to_owned());
    idx2.insert("changedid".to_owned(),"SHA2&2".to_owned());
    // These records have been added
    idx2.insert("addedid".to_owned(),"SHA1&3".to_owned());
    let (added, modified, deleted) = calculate_diff(&mut idx1,&mut idx2,verbosity);
    assert_eq!(added.len(),1);
    assert_eq!(modified.len(),1);
    assert_eq!(deleted.len(),1);
  }
}
