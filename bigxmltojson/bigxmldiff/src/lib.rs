//! BigXMLdiff Crate
//! The Difference is calculated as follows:
//! An first XML file is read and split into the interested `<ITEM>s`
//! A `unique_id` must be created from each the <ITEM>, even by composition.
//! The key is constructed of flat structs. <KEY_0>Something</KEY_0>
//! A checksum of the `<ITEM>` content is calculated.
//! A map of `unique_id` to checksum is stored in a sorted "checksum" file.
//! The second XML is read and split into `<ITEM>s`
//! The `unique_id` is calculated following same process as above.
//! The checksum is calculated for the content of the <ITEM>.
//! If the checksum differs or does not exist the difference is returned.
//! Just for simplicity in the final analysis, the result is shown in JSON format.
//! Caveats:
//! This is not an XML validator, use xmllint --noout --validate xml1 xml2
//!
#![feature(test)] 
extern crate test;
extern crate crypto;
extern crate getopts;

pub mod config;

use self::crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::path::Path;

// Read 128K bytes, the XMLs of our <ITEM>s are around 20Ks usually.
const CAP: usize = 1024 * 128;
/// `get_id` calcutes an ID based on the config in the struct 
/// returns true if all keys are found.
pub fn get_id(data: &String, xml_keys: &Vec<String>, return_key: &mut Vec<String>) -> bool {
  for cur_key in return_key.into_iter() {
    cur_key.truncate(0);
  }
  let mut inside_tag = false;
  let mut cur_tag = String::with_capacity(128);
  let mut cur_tag_content = String::with_capacity(128);
  let mut found_keys = 0usize;
  for cur_char in data.chars() {
    if cur_char == '<' {
      inside_tag = true;
      continue;
    }
    if cur_char == '>' {
      for (ith, key) in xml_keys.iter().enumerate() {
        if cur_tag.eq(key) {
          return_key[ith].push_str(&cur_tag_content);
          //println!("Found tag '{}': {} -> {}", cur_tag, cur_tag_content,return_key[ith]);
          found_keys += 1;
        }
      }
      cur_tag_content.truncate(0);
      cur_tag.truncate(0);
      inside_tag = false;
      continue;
    }
    if inside_tag {
      cur_tag.push(cur_char);
      // println!("Adding to cur_tag: '{}': {}", cur_char, cur_tag);
    } else {
      cur_tag_content.push(cur_char);
      // println!("Adding to cur_tag_content: '{}': {}", cur_char, cur_tag_content);
    }
  }
  found_keys == xml_keys.len()
}

/// `get_xml_chunk`: Returns a chunk that matches a specific delimiter_tag contents
/// It alters the input data to remove prelude and returns a Some(String) if the delimited chunk is
/// found.
/// It returns None if the chunk desired is not found.
pub fn get_xml_chunk(data: &mut String, delimiter_tag: &String) -> Option<String> {
  let mut start_tag = "<".to_owned();
  start_tag.push_str(delimiter_tag);
  start_tag.push_str(">");
  let mut end_tag = "</".to_owned();
  end_tag.push_str(delimiter_tag);
  end_tag.push_str(">");
  match data.find(&start_tag) {
    Some(start) => {
      match data.find(&end_tag) {
        Some(end) => {
          // remove the starting <TAG> itself.
          data.drain(.. (start + start_tag.len()));
          // remove the ending </TAG>
          let new_end = end - start - start_tag.len();
          data.drain(new_end .. (new_end + end_tag.len()));
          Some(data.drain(..new_end).collect())
        },
        None => None
      }
    },
    None => None
  }
}

/// `calculate_sha256_on_chunk` gets a data chunk and creates a SHA256 out of it.
pub fn calculate_sha256_on_chunk(input: &String) -> String {
  let mut sha = Sha256::new();
  // Sort the lines, in case the items/lines shift.
  let mut lines = input.lines().sort();
  sha.input_str(lines.join("\n"));
  sha.result_str()
}

/// `parse_data_chunk` gets a data chunk and creates a SHA256 out of it.
/// The offset is needed to go back to the file and identify the chunk.
pub fn parse_data_chunk(data_chunk: &mut String, offset: usize, cfg: &Config) -> (String,String) {
  while let Some(xml_chunk) = get_xml_chunk(&mut data_chunk, cfg.delimiter_tag) {
    (calculate_sha256_on_chunk(xml_chunk),offset)
  }
}

/// `read_file_in_chunks`: BufReader's the file into CAP sized data chunks.
/// The data chunks are checked for XML chunks that can be further parsed.
pub fn read_file_in_chunks(filename: &String, delimiter_tag: &String, xml_keys: &Vec<String>) {
  let file = File::open(&Path::new(filename)).unwrap();
  let mut reader = BufReader::with_capacity(CAP, file);
  let mut xml_chunk = String::with_capacity(CAP * 2);
  loop {
    let length = {
      let mut buffer = try!(reader.fill_buf());
      // Get one of our XML subsets from the buffer.
      xml_chunk.push_str(buffer);
      buffer.len();
    };
    if length == 0 {
      break;
    }
    reader.consume(length);
  }
}

// Creates a checksum file of an XML.
pub fn build_checksum_map(cfg: &Config) {
  read_file_in_chunks(cfg);
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::{Bencher,black_box};

  #[test]
  fn it_gets_id() {
    let xml_keys: Vec<String> = vec![
      "/KEY_1".to_owned(),
      "/KEY_2".to_owned(),
      "/KEY_3".to_owned(),
    ];
    let mut xml_key: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(get_id(&test_xml,&xml_keys,&mut xml_key),true);
    assert_eq!(xml_key.join(":"),"1::A".to_owned());
    let test_xml = "
    
    <KEY_1>1</KEY_1>
      <KEY_3>A</KEY_3>
      <KEY_2></KEY_2>".to_owned();
    assert_eq!(get_id(&test_xml,&xml_keys,&mut xml_key),true);
    assert_eq!(xml_key.join(":"),"1::A".to_owned());
    let test_xml = "NotAnXML".to_owned();
    assert_eq!(get_id(&test_xml,&xml_keys,&mut xml_key),false);
    let test_xml = "<KEY_3>A</KEY_3>".to_owned();
    assert_eq!(get_id(&test_xml,&xml_keys,&mut xml_key),false);
  }
  #[test]
  fn it_gets_chunks() {
    let mut test_xml = "<PRELUDE_TAGS></PRELUDE_TAGS>
      <IMPORTANT_DATA><INTERNAL_DATA>A</INTERNAL_DATA></IMPORTANT_DATA>
      <IRRELEVANT_DATA>1</IRRELEVANTDATA>
      <IMPORTANT_DATA>".to_owned();
    assert_eq!(get_xml_chunk(&mut test_xml, &"IMPORTANT_DATA".to_owned()),Some("<INTERNAL_DATA>A</INTERNAL_DATA>".to_owned()));
    assert_eq!(test_xml,"
      <IRRELEVANT_DATA>1</IRRELEVANTDATA>
      <IMPORTANT_DATA>".to_owned());
    // There is not enough data left in this XML for a complete chunk:
    assert_eq!(get_xml_chunk(&mut test_xml, &"IMPORTANT_DATA".to_owned()),None);
    // Add more data that completes the partial chunk and contains a new chunk:
    test_xml.push_str("<INTERNAL_DATA>B</INTERNAL_DATA></IMPORTANT_DATA><IMPORTANT_DATA><INTERNAL_DATA>C</INTERNAL_DATA></IMPORTANT_DATA></LAST_TAG>");
    assert_eq!(get_xml_chunk(&mut test_xml, &"IMPORTANT_DATA".to_owned()),Some("<INTERNAL_DATA>B</INTERNAL_DATA>".to_owned()));
    assert_eq!(test_xml,"<IMPORTANT_DATA><INTERNAL_DATA>C</INTERNAL_DATA></IMPORTANT_DATA></LAST_TAG>".to_owned());
    assert_eq!(get_xml_chunk(&mut test_xml, &"IMPORTANT_DATA".to_owned()),Some("<INTERNAL_DATA>C</INTERNAL_DATA>".to_owned()));
    assert_eq!(test_xml,"</LAST_TAG>".to_owned());
  }
  #[test]
  fn it_gets_sha256() {
    let test_xml = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(calculate_sha256_on_chunk(&test_xml),calculate_sha256_on_chunk(&test_xml));
  }
  #[bench]
  fn bench_get_id(b: &mut Bencher) {
    let xml_keys: Vec<String> = vec![
      "/KEY_1".to_owned(),
      "/KEY_2".to_owned(),
      "/KEY_3".to_owned(),
    ];
    let mut xml_key: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      get_id(&test_xml,&xml_keys,&mut xml_key)
    });
   }
  #[bench]
  fn bench_get_id_blackbox(b: &mut Bencher) {
    let xml_keys: Vec<String> = vec![
      "/KEY_1".to_owned(),
      "/KEY_2".to_owned(),
      "/KEY_3".to_owned(),
    ];
    let mut xml_key: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      for _ in 1..10000 {
        black_box(get_id(&test_xml,&xml_keys,&mut xml_key));
      }
    });
   }
}
