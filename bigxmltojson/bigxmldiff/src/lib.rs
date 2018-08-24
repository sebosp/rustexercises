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
extern crate zmq;
extern crate rand;

pub mod config;
pub mod chunkindex;
use config::*;
use chunkindex::*;

use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::str;
use zmq::SNDMORE;
use std::time::Instant;
use std::thread;

/// `hex` transforms a Vector of u8 into a hex string.
fn hex(bytes: &[u8]) -> String {
  bytes.iter().map(|x| format!("{:02x}", x)).collect::<Vec<_>>().join("")
}

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
  found_keys == xml_keys.len()
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

/// `process_chunk` Receives an xml_chunk and gets the ID and SHA from it.
/// It stores the ID, SHA and Offset into the chunk_index BTreeMap.
/// Returns the chunk_id, the shasum and the record_offset.
pub fn process_chunk(xml_chunk: &String,
                     chunk_id: &mut Vec<String>,
                     xml_keys: &Vec<String>,
                     record_offset: usize,
                     num_record: usize,
) -> Result<(String,String,usize),String>{
  if get_id(&xml_chunk,xml_keys,chunk_id) {
    let checksum = calculate_checksum(&xml_chunk);
    println!("Record {} id: {} shasum {}",num_record, chunk_id.join("&"),checksum);
    Ok((chunk_id.join("&"),checksum,record_offset))
  } else {
    return Err(format!("Unable to find key for chunk at offset {}",record_offset))
  }
}

fn worker_task(xml_keys: Vec<String>, connect_endpoint: String) {
  let context = zmq::Context::new();
  let worker = context.socket(zmq::DEALER).unwrap();
  let identity: Vec<_> = (0..10).map(|_| rand::random::<u8>()).collect();
  worker.set_identity(&identity).expect("failed setting client id");
  worker.connect(&connect_endpoint.replace("*","localhost")).expect("failed connecting client");

  let mut total = 0;
  let mut chunk_id: Vec<String> = vec![];
  for _ in 0 .. xml_keys.len() {
    // Reserve memory for our key XXX: Magic number.
    chunk_id.push(String::with_capacity(64));
  }
  // Tell the broker we're ready for work
  worker.send(b"", SNDMORE).unwrap(); // Envelope
  worker.send(b"READYFORWORK", 0).unwrap();
  loop {

    // Get workload from broker
    worker.recv_bytes(0).unwrap();  // envelope delimiter
    let workload = worker.recv_string(0).unwrap().unwrap();
    if workload == "ENDOFWORKLOAD" {
      println!("Worker {} completed {} tasks", hex(&identity), total);
      break;
    }
    let xml_chunk = worker.recv_string(0).unwrap().unwrap();
    println!("WORKER: Got xml_chunk: {:?}",xml_chunk);
    let record_offset = worker.recv_string(0).unwrap().unwrap().parse::<usize>().unwrap();
    let num_record = worker.recv_string(0).unwrap().unwrap().parse::<usize>().unwrap();
    let (processed_id, processed_checksum, processed_offset) = process_chunk(&xml_chunk,
                  &mut chunk_id,
                  &xml_keys,
                  record_offset,
                  num_record).unwrap();
    worker.send(b"", SNDMORE).unwrap();
    worker.send(b"RESULT", SNDMORE).unwrap();
    worker.send(&processed_id.into_bytes(), SNDMORE).unwrap();
    worker.send(&processed_checksum.to_string().into_bytes(), SNDMORE).unwrap();
    worker.send(&processed_offset.to_string().into_bytes(), 0).unwrap();
    total += 1;
  }
}
pub fn process_response(broker: &zmq::Socket, chunk_index: &mut ChunkIndex) -> Result<i8,String> {
  let response_type_raw = &broker.recv_bytes(0).unwrap();
  let response_type = str::from_utf8(response_type_raw).unwrap();
  println!("BROKER: response_type: {:?}",response_type);
  if response_type == "READYFORWORK" {
    return Ok(1);
  }
  let processed_id_raw = &broker.recv_bytes(0).unwrap();
  let processed_id = str::from_utf8(processed_id_raw).unwrap();
  println!("BROKER: got processed_id: {:?}",processed_id);
  let processed_checksum_raw = &broker.recv_bytes(0).unwrap();
  println!("processed_checksum: {:?}",processed_checksum_raw);
  let processed_checksum = str::from_utf8(processed_checksum_raw).unwrap();
  let processed_offset_raw = &broker.recv_bytes(0).unwrap();
  let processed_offset = str::from_utf8(processed_offset_raw).unwrap()
    .parse::<usize>()
    .expect("Could not parse processed_offset");
  println!("BROKER: Inserting to chunk_index");
  if ! chunk_index.insert(processed_id.clone().to_string(),format!("{}&{}", processed_checksum, processed_offset)) {
    let prev_payload = match chunk_index.search(&processed_id.to_string()) {
      Some(payload) => payload,
      None => "Unset", // OOM?
    };
    return Err(format!("At offset {}, found existing key {} at sha&offset {}",processed_offset,processed_id,prev_payload))
  }
  Ok(-1)
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
  // We need to add <DELIM_TAG></DELIM_TAG> to the offset to account for proper offset.
  // These are removed from the get_xml_chunk function.
  // We take the first character after the opening <DELIM_TAG> as the offset of 
  // the chunk. Later we also need to account for the 3 chars </> of the DELIM TAG:
  let delim_key_size:usize = cfg.chunk_delimiter.len() + 3;
  // ZMQ setup
  let context = zmq::Context::new();
  let broker = context.socket(zmq::ROUTER).unwrap();
  println!("Binding to: {}",&cfg.bind_address);
  assert!(broker.bind(&cfg.bind_address).is_ok());
  let mut thread_pool = Vec::new();
  for _ in 0 .. cfg.concurrency {
    let xml_keys = cfg.xml_keys.clone();
    let connect_endpoint = cfg.bind_address.clone();
    let child = thread::spawn(move || {
      worker_task(xml_keys,connect_endpoint);
    });
    thread_pool.push(child);
  }
  println!("Starting {} threads",cfg.concurrency);
  let start_time = Instant::now();
  let mut concurrent_requests = 0i8;
  let mut event_items = [
    broker.as_poll_item(zmq::POLLIN),
  ];
  let mut max_retries = 100_000;
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

        max_retries = 100_000;
        loop {
          zmq::poll(&mut event_items, 1000).expect("client failed polling");
          if ! event_items[0].is_readable() {
            max_retries -= 1;
            continue;
          }
          if max_retries < 1 {
            break;
          }
        }
        if max_retries == 0 {
          return Err("Max retries exceeded waiting for clients to return work".to_owned());
        }
        let worker_id = broker.recv_bytes(0).unwrap(); // ID frame
        println!("BROKER: Got worker id: {:?}",worker_id);
        concurrent_requests += process_response(&broker,&mut chunk_index)?;
        broker.send(&worker_id, SNDMORE).unwrap();
        broker.send(b"", SNDMORE).unwrap();
        broker.send(b"REQUEST", SNDMORE).unwrap();
        broker.send(&xml_chunk.into_bytes(), SNDMORE).unwrap();
        broker.send(&record_offset.to_string().into_bytes(), SNDMORE).unwrap();
        broker.send(&num_records.to_string().into_bytes(), 0).unwrap();
        println!("Sent request to worker thread");
        if concurrent_requests / 2 <= cfg.concurrency {
          break;
        }
      }
      buffer.len()
    };
    if length == 0 {
      break;
    }
    reader.consume(length);
  }
  loop {
   if concurrent_requests == 0 || max_retries == 0 {
     break;
   }
   //if broker.poll(zmq::POLLIN, 1000).expect("client failed polling") > 0 {
   zmq::poll(&mut event_items, 1000).expect("client failed polling");
   if event_items[0].is_readable() {
     println!("Pollin.");
     concurrent_requests += process_response(&broker,&mut chunk_index)?;
   }
   max_retries -= 1;
  }
  for _ in 0 .. cfg.concurrency {
    broker.send(b"ENDOFWORKLOAD", 0).unwrap();
  }
  println!("Finished after {}", start_time.elapsed().as_secs());
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
      10i8,
      "tcp://*:5555".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "
    
    <KEY_1>1</KEY_1>
      <KEY_3>A</KEY_3>
      <KEY_2></KEY_2>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "NotAnXML".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id),false);
    let test_xml = "<KEY_3>A</KEY_3>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id),false);
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
    );
    let mut html_chunk_id: Vec<String> = vec![
      String::with_capacity(64),
    ];
    let test_html = "<html><head></head> <body> <div><li>1</li></div> </body> </html>".to_owned();
    assert_eq!(get_id(&test_html,&cfg.xml_keys,&mut html_chunk_id),true);
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
        10i8,
        "tcp://*:5555".to_owned(),
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
      10i8,
      "tcp://*:5555".to_owned(),
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
      10i8,
      "tcp://*:5671".to_owned(),
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
      10i8,
      "tcp://*:5555".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      get_id(&test_xml,&cfg.xml_keys,&mut chunk_id)
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
      10i8,
      "tcp://*:5555".to_owned(),
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      for _ in 1..1000 {
        black_box(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id));
      }
    });
   }
  // Need to find another way to bench this. read_file_in_chunks binds to a port.
  #[bench]
  #[ignore]
  fn bench_read_smallfile_smallchunks(b: &mut Bencher) {
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "tests/test1.xml".to_owned(),
      10usize,
      "checksum".to_owned(),
      1i8,
      "tcp://*:5555".to_owned(),
    );
    b.iter(|| {
      for _ in 1..1000 {
        black_box(read_file_in_chunks(&cfg).unwrap());
      }
    });
  }
}
