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
//! It's assumed there are no comments (`<!-- -->`) or `CDATA[ sections ]`.
//!
#![feature(test)] 
extern crate test;
extern crate getopts;
extern crate crypto;

pub mod config;
pub mod chunkindex;
use config::*;
use chunkindex::*;

use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::str;

/// `get_id` calcutes an ID based on the config in the struct 
/// returns true if all keys are found.
pub fn get_id(data: &String, cfg: &Config, return_key: &mut Vec<String>) -> bool {
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
      for (ith, key) in cfg.xml_keys.iter().enumerate() {
        if cur_tag.eq(key) {
          return_key[ith].push_str(&cur_tag_content);
          // println!("Found key '{}': {} -> {}", cur_tag, cur_tag_content,return_key.join("&"));
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
  found_keys == cfg.xml_keys.len()
}

/// `get_xml_chunk`: Returns a chunk that matches a specific delimiter_tag contents
/// It alters the input data to remove prelude and returns a Some(String,Offset)
/// if the delimited chunk is found. The Offset is the start of the XML in the current chunk.
/// It returns None if the chunk desired is not found.
pub fn get_xml_chunk(data: &mut String, cfg: &Config) -> Option<(String,usize)> {
  let start_tag = format!("<{}>", &cfg.chunk_delimiter);
  let end_tag = format!("</{}>", &cfg.chunk_delimiter);
  match data.find(&start_tag) {
    Some(start) => {
      match data.find(&end_tag) {
        Some(end) => {
          // remove the starting <TAG> itself.
          data.drain(.. (start + start_tag.len()));
          // remove the ending </TAG>
          let new_end = end - start - start_tag.len();
          data.drain(new_end .. (new_end + end_tag.len()));
          Some((data.drain(..new_end).collect(),start + start_tag.len()))
        },
        None => None
      }
    },
    None => None
  }
}

pub fn process_chunk(xml_chunk: &String,
                     cfg: &Config,
                     chunk_id: &mut Vec<String>,
                     chunk_offset: usize,
                     num_record: usize,
                     chunk_index: &mut ChunkIndex
) -> Result<(),String>{
  if get_id(&xml_chunk,&cfg,chunk_id) {
    println!("Record {} id: {}",num_record, chunk_id.join("&"));
    if ! chunk_index.insert(chunk_id.join("&"),format!("{}&{}",calculate_checksum(&xml_chunk),chunk_offset)) {
      let prev_payload = match chunk_index.search(&chunk_id.join("&")) {
        Some(payload) => payload,
        None => "Unset", // OOM?
      };
      return Err(format!("At offset {}, found existing key {} at sha&offset {}",chunk_offset,chunk_id.join("&"),prev_payload))
    }
  } else {
    return Err(format!("Unable to find key for chunk at offset {}",chunk_offset))
  }
  Ok(())
}

/// `read_file_in_chunks`: BufReader's the file into CAP sized data chunks.
/// The data chunks are checked for XML chunks that can be further parsed.
pub fn read_file_in_chunks(cfg: &Config) -> Result<usize, String> {
  println!("Checking file {}.",&cfg.input_filename);
  let file = File::open(Path::new(&cfg.input_filename)).unwrap();
  let mut reader = BufReader::with_capacity(cfg.chunk_size, file);
  let mut data_chunk = String::with_capacity(cfg.chunk_size * 2);
  let mut num_records = 0usize;
  let mut offset = 0usize;
  let mut chunk_index = ChunkIndex::new(&cfg.input_filename);
  let mut chunk_id: Vec<String> = vec![];
  // We need to add <DELIM_TAG></DELIM_TAG> to the offset to account for proper offset.
  // These are removed from the get_xml_chunk function.
  // We take the first character after the opening <DELIM_TAG> as the offset of 
  // the chunk. Later we also need to account for the 3 chars </> of the DELIM TAG:
  let delim_key_size:usize = cfg.chunk_delimiter.len() + 3;
  for _ in 0 .. cfg.xml_keys.len()  {
    // Reserve memory for our key
    chunk_id.push(String::with_capacity(64));
  }
  loop {
    let length = {
      let mut buffer = reader.fill_buf().unwrap();
      // Get one of our XML subsets from the buffer.
      let buffer_string = match str::from_utf8(buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
      };
      data_chunk += buffer_string;
      while let Some((xml_chunk, chunk_offset)) = get_xml_chunk(&mut data_chunk, &cfg) {
        num_records+=1;
        offset += chunk_offset + xml_chunk.len() + delim_key_size;
        let record_offset = offset - xml_chunk.len() - delim_key_size;
        process_chunk(&xml_chunk,
                      &cfg,
                      &mut chunk_id,
                      record_offset,
                      num_records,
                      &mut chunk_index)?;
      }
      buffer.len()
    };
    if length == 0 {
      break;
    }
    reader.consume(length);
  }
  match chunk_index.store(&format!("{}.idx",&cfg.input_filename)) {
    Ok(_)    => Ok(num_records),
    Err(err) => Err(format!("Unable to write index file: {}",err)),
  }
}

// Creates a chunk index file from an XML
pub fn build_chunkindex_from_xml(cfg: &Config) {
  match read_file_in_chunks(cfg) {
    Ok(num_records) => {
      println!("Consumed {} records", num_records);
    },
    Err(err) => {
      println!("Error processing file: {}",err);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::{Bencher,black_box};

  #[test]
  fn it_gets_id() {
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg,&mut chunk_id),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "
    
    <KEY_1>1</KEY_1>
      <KEY_3>A</KEY_3>
      <KEY_2></KEY_2>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg,&mut chunk_id),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "NotAnXML".to_owned();
    assert_eq!(get_id(&test_xml,&cfg,&mut chunk_id),false);
    let test_xml = "<KEY_3>A</KEY_3>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg,&mut chunk_id),false);
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    let mut html_chunk_id: Vec<String> = vec![
      String::with_capacity(64),
    ];
    let test_html = "<html><head></head> <body> <div><li>1</li></div> </body> </html>".to_owned();
    assert_eq!(get_id(&test_html,&cfg,&mut html_chunk_id),true);
    assert_eq!(html_chunk_id.join("&"),"1".to_owned());
  }
  #[test]
  fn it_gets_chunks() {
    let cfg = Config::new(
        "INVALID".to_owned(),
        "IMPORTANT_DATA".to_owned(),
        "MEMORY".to_owned(),
        50usize,
        "test".to_owned(),
    );
    let mut test_xml = "<PRELUDE_TAGS></PRELUDE_TAGS>
      <IMPORTANT_DATA><INTERNAL_DATA>A</INTERNAL_DATA></IMPORTANT_DATA>
      <IRRELEVANT_DATA>1</IRRELEVANTDATA>
      <IMPORTANT_DATA>".to_owned();
    assert_eq!(get_xml_chunk(&mut test_xml, &cfg),Some(("<INTERNAL_DATA>A</INTERNAL_DATA>".to_owned(),52usize)));
    assert_eq!(test_xml,"
      <IRRELEVANT_DATA>1</IRRELEVANTDATA>
      <IMPORTANT_DATA>".to_owned());
    // There is not enough data left in this XML for a complete chunk:
    assert_eq!(get_xml_chunk(&mut test_xml, &cfg),None);
    // Add more data that completes the partial chunk and contains a new chunk:
    test_xml.push_str("<INTERNAL_DATA>B</INTERNAL_DATA></IMPORTANT_DATA><IMPORTANT_DATA><INTERNAL_DATA>C</INTERNAL_DATA></IMPORTANT_DATA></LAST_TAG>");
    assert_eq!(get_xml_chunk(&mut test_xml, &cfg),Some(("<INTERNAL_DATA>B</INTERNAL_DATA>".to_owned(),65usize)));
    assert_eq!(test_xml,"<IMPORTANT_DATA><INTERNAL_DATA>C</INTERNAL_DATA></IMPORTANT_DATA></LAST_TAG>".to_owned());
    assert_eq!(get_xml_chunk(&mut test_xml, &cfg),Some(("<INTERNAL_DATA>C</INTERNAL_DATA>".to_owned(),16usize)));
    assert_eq!(test_xml,"</LAST_TAG>".to_owned());
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    let mut test_html = "<html><head></head> <body> <div><li>1</li></div> <div><li>2</li></div> <div><li>3</li></div> </body> </html>".to_owned();
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>1</li>".to_owned(),32usize)));
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>2</li>".to_owned(),6usize)));
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>3</li>".to_owned(),6usize)));
  }
  #[test]
  fn it_adds_chunks() {
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "tests/test1-dup.xml".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    // There are 2 duplicate IDs in chunks in this file. in the same line.
    // <div><li>1</li></div><div><li>1</li></div>
    // 01234^67890123456789012345^7890
    let duplicate_id_tag = "<li>1</li>".to_owned();
    let id_sha = calculate_checksum(&duplicate_id_tag);
    assert_eq!(
      read_file_in_chunks(&cfg),
      Err(format!("At offset {}, found existing key {} at sha&offset {}&{}",26usize,1,id_sha,5usize))
    );
  }
  #[bench]
  fn bench_get_id(b: &mut Bencher) {
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      get_id(&test_xml,&cfg,&mut chunk_id)
    });
   }
  #[bench]
  fn bench_get_id_blackbox(b: &mut Bencher) {
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      for _ in 1..1000 {
        black_box(get_id(&test_xml,&cfg,&mut chunk_id));
      }
    });
   }
  #[bench]
  fn bench_read_smallfile_smallchunks(b: &mut Bencher) {
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "tests/test1.xml".to_owned(),
      10usize,
      "checksum".to_owned(),
    );
    b.iter(|| {
      for _ in 1..1000 {
        black_box(read_file_in_chunks(&cfg).unwrap());
      }
    });
  }
}
