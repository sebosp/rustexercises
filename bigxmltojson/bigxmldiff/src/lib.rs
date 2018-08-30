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
pub fn get_id(data: &String, xml_keys: &Vec<String>, return_key: &mut Vec<String>, worker_id_string: &String, verbosity: i8) -> bool {
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
          if verbosity > 3 {
            println!("WORKER[{}] Found key '{}': {} -> {}", worker_id_string, cur_tag, cur_tag_content,return_key.join("&"));
          }
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
      if verbosity > 3 {
        println!("WORKER[{}] Adding to cur_tag: '{}': {}", worker_id_string, cur_char, cur_tag);
      }
    } else {
      cur_tag_content.push(cur_char);
      if verbosity > 3 {
        println!("WORKER[{}] Adding to cur_tag_content: '{}': {}", worker_id_string, cur_char, cur_tag_content);
      }
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
                     worker_id_string: &String,
                     verbosity: i8,
) -> Result<(String,String),String>{
  if get_id(&xml_chunk,xml_keys,chunk_id, &worker_id_string,verbosity) {
    let checksum = calculate_checksum(&xml_chunk);
    Ok((chunk_id.join("&"),checksum))
  } else {
    return Err("get_id: Unable to find key".to_owned());
  }
}

pub fn to_string(socket: &zmq::Socket) -> Result<String,String> {
  let data = socket
    .recv_string(0)
    .expect("Unable to recv data")
    .expect("Unable to transform to string");
  Ok(data)
}
fn worker_task(xml_keys: Vec<String>, connect_endpoint: String, verbosity: i8) {
  let context = zmq::Context::new();
  let worker = context.socket(zmq::DEALER).unwrap();

  let start_time = Instant::now();
  let identity: Vec<_> = (0..10).map(|_| rand::random::<u8>()).collect();
  worker.set_identity(&identity).expect("failed setting client id");
  let worker_id_string = hex(&identity);

  // Replace * to localhost in case the connection broker binds to all addresses.
  worker.connect(&connect_endpoint.replace("*","localhost")).expect("failed connecting client");
  let max_tasks = 10000;
  let mut total_tasks = 0;
  let mut chunk_id: Vec<String> = vec![];
  for _ in 0 .. xml_keys.len() {
    // Reserve memory for our key XXX: Magic number.
    chunk_id.push(String::with_capacity(64));
  }
  // Tell the broker we're ready for work
  if verbosity > 0 {
    println!("WORKER[{}] Registering to broker", &worker_id_string);
  }
  worker.send(b"", SNDMORE).unwrap(); // Envelope
  worker.send(b"READYFORWORK", 0).unwrap();
  let mut worktype:String;
  let mut xml_chunk:String ;
  let mut record_offset:String;
  let mut num_record:String;
  let mut processed_id:String;
  let mut processed_checksum:String;
  let mut chunk_size:usize;
  loop {

    // Get worktype from broker
    worker.recv_bytes(0).unwrap();  // envelope delimiter
    worktype = worker.recv_string(0).unwrap().unwrap();
    if verbosity > 1 {
      println!("WORKER[{}] Got worktype: {:?}", &worker_id_string, worktype);
    }
    if worktype == "ENDOFWORKLOAD" {
      if verbosity > 0 {
        println!("WORKER[{}] completed {} tasks", &worker_id_string, total_tasks);
      }
      break;
    }
    xml_chunk = worker.recv_string(0).unwrap().expect("Unable to receive xml_chunk");
    chunk_size = xml_chunk.len();
    if verbosity > 2 {
      println!("WORKER[{}] Got xml_chunk: {:?} with size: {}", &worker_id_string, xml_chunk, chunk_size);
    }
    record_offset = worker.recv_string(0).unwrap().expect("Unable to get record_offset");
    num_record = worker.recv_string(0).unwrap().expect("Unable to get record number");
    match process_chunk(&xml_chunk,
                  &mut chunk_id,
                  &xml_keys,
                  &worker_id_string,
                  verbosity) {
      Ok((proc_id, proc_chksum)) => {
        processed_id = proc_id;
        processed_checksum = proc_chksum;
      },
      Err(err)                   => {
        println!("At record offset {}: {}",record_offset, err);
        break;
      }
    }
    if verbosity > 2 {
      println!("WORKER[{}] Record {} id: {} shasum {}", &worker_id_string ,num_record, processed_id, processed_checksum);
    }
    worker.send(b"", SNDMORE).unwrap();
    if total_tasks > max_tasks {
      worker.send(b"LASTRESULT", SNDMORE).unwrap(); // Worker is finishing.
    } else {
      worker.send(b"RESULT", SNDMORE).unwrap();
    }
    worker.send(&processed_id.into_bytes(), SNDMORE).unwrap();
    worker.send(&processed_checksum.into_bytes(), SNDMORE).unwrap();
    worker.send(&record_offset.into_bytes(), 0).unwrap();
    if total_tasks > max_tasks {
      if verbosity > 0 {
        println!("WORKER[{}] completed {} tasks in {} seconds", &worker_id_string, total_tasks, start_time.elapsed().as_secs());
      }
      break;
    }
    total_tasks += 1;
  }
}
/// `process_response` checks for more data from the worker.
/// It returns the worker increase.
/// If a worker replies with "READYFORWORK", we increment the amount of workers.
pub fn process_response(broker: &zmq::Socket, chunk_index: &mut ChunkIndex, verbosity: i8) -> Result<i8,String> {
  let response_type_raw = &broker.recv_bytes(0).unwrap();
  let response_type = str::from_utf8(response_type_raw).unwrap();
  if verbosity > 2 {
    println!("BROKER: response_type: {:?}",response_type);
  }
  if response_type == "READYFORWORK" {
    return Ok(1);
  }
  let processed_id_raw = &broker.recv_bytes(0).unwrap();
  let processed_id = str::from_utf8(processed_id_raw).unwrap();
  if verbosity > 2 {
    println!("BROKER: got processed_id: {:?}",processed_id);
  }
  let processed_checksum_raw = &broker.recv_bytes(0).unwrap();
  let processed_checksum = str::from_utf8(processed_checksum_raw).unwrap();
  let processed_offset_raw = &broker.recv_bytes(0).unwrap();
  let processed_offset = str::from_utf8(processed_offset_raw).unwrap()
    .parse::<usize>()
    .expect("Could not parse processed_offset");
  if verbosity > 1 {
    println!("BROKER: Inserting to chunk_index");
    println!("{}&{}&{}", processed_id, processed_checksum, processed_offset);
  }
  if ! chunk_index.insert(processed_id.clone().to_string(),format!("{}&{}", processed_checksum, processed_offset)) {
    let prev_payload = match chunk_index.search(&processed_id.to_string()) {
      Some(payload) => payload,
      None => "Unset", // OOM?
    };
    return Err(format!("At offset {}, found existing key {} at sha&offset {}",processed_offset,processed_id,prev_payload))
  }
  if response_type == "LASTRESULT" {
     Ok(-1)
  } else {
    Ok(0)
  }
}

/// `read_file_in_chunks`: BufReader's the file into CAP sized data chunks.
/// The data chunks are checked for XML chunks that can be further parsed.
pub fn read_file_in_chunks(cfg: &Config, filename: &String) -> Result<(ChunkIndex, usize), String> {
  println!("Checking file {}.",&filename);
  let file = File::open(Path::new(&filename)).unwrap();
  let mut reader = BufReader::with_capacity(cfg.chunk_size, file);
  let mut data_chunk = String::with_capacity(cfg.chunk_size * 2);
  let mut num_records = 0usize;
  let mut offset = 0usize;
  let mut chunk_index = ChunkIndex::new(&filename);
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
  let mut chunk_size:usize;
  let max_tasks = 1000_000;
  for _ in 0 .. cfg.concurrency {
    let xml_keys = cfg.xml_keys.clone();
    let connect_endpoint = cfg.bind_address.clone();
    let verbosity = cfg.verbosity.clone();
    thread::spawn(move || {
      worker_task(xml_keys,connect_endpoint,verbosity);
    });
  }
  println!("Starting {} threads",cfg.concurrency);
  let start_time = Instant::now();
  let mut concurrent_requests = 0i8;
  let mut event_items = [
    broker.as_poll_item(zmq::POLLIN),
  ];
  let mut max_retries:i64;
  let mut worker_number_delta:i8;
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
          if event_items[0].is_readable() {
            break;
          }
          if max_retries < 1 {
            break;
          }
        }
        if max_retries < 1 {
          return Err("Max retries exceeded waiting for clients to return work".to_owned());
        }
        let worker_id = broker.recv_bytes(0).unwrap(); // ID frame
        broker.recv_bytes(0).unwrap(); // Delimiter frame
        if cfg.verbosity > 0 {
          println!("BROKER: Got worker id: {}",hex(&worker_id));
        }
        worker_number_delta = process_response(&broker,&mut chunk_index,cfg.verbosity)?;
        if worker_number_delta == -1 {
          // Spawn a new thread in case one worker disconnects
          let xml_keys = cfg.xml_keys.clone();
          let connect_endpoint = cfg.bind_address.clone();
          let verbosity = cfg.verbosity.clone();
          thread::spawn(move || {
            worker_task(xml_keys,connect_endpoint,verbosity);
          });
        }
        concurrent_requests += worker_number_delta;
        broker.send(&worker_id, SNDMORE).unwrap();
        broker.send(b"", SNDMORE).unwrap();
        broker.send(b"PROCESS_CHUNK", SNDMORE).unwrap();
        chunk_size = xml_chunk.len();
        broker.send(&xml_chunk.into_bytes(), SNDMORE).unwrap();
        broker.send(&record_offset.to_string().into_bytes(), SNDMORE).unwrap();
        broker.send(&num_records.to_string().into_bytes(), 0).unwrap();
        if cfg.verbosity > 2 {
          println!("BROKER: Sent Chunk Sized {} request to worker thread", chunk_size);
        }
      }
      if num_records % 50000 == 0 {
        println!("BROKER: Processed {} in {} seconds",num_records,start_time.elapsed().as_secs());
      }
      buffer.len()
    };
    if length == 0 {
      break;
    }
    reader.consume(length);
    if num_records > max_tasks {
      break;
    }
  }
  max_retries = 100_000;
  loop {
   if concurrent_requests == 0 || max_retries == 0 {
     break;
   }
   zmq::poll(&mut event_items, 1000).expect("client failed polling");
   if event_items[0].is_readable() {
     let worker_id = broker.recv_bytes(0).unwrap(); // ID frame
     broker.recv_bytes(0).unwrap(); // Delimiter frame
     if cfg.verbosity > 2 {
       println!("BROKER: Got worker id: {}",hex(&worker_id));
     }
     process_response(&broker,&mut chunk_index,cfg.verbosity)?;
     broker.send(&worker_id, SNDMORE).unwrap();
     broker.send(b"", SNDMORE).unwrap();
     broker.send(b"ENDOFWORKLOAD", SNDMORE).unwrap();
     concurrent_requests -= 1;
   }
   max_retries -= 1;
  }
  println!("Finished after {}", start_time.elapsed().as_secs());
  match chunk_index.store(&format!("{}.idx",&filename)) {
    Ok(_)    => Ok((chunk_index, num_records)),
    Err(err) => Err(format!("Unable to write index file: {}",err)),
  }
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
pub fn calculate_diff(cfg: &Config, idx1: &mut ChunkIndex, idx2: &mut ChunkIndex) -> (Vec<usize>,Vec<usize>,Vec<usize>) {
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
          if cfg.verbosity > 0 {
            println!("+ modified: {}: (prev: {}, new: {})",chunk1_id,chunk1_payload[0],chunk2_payload[0]);
          }
          offset = chunk2_payload[1].parse::<usize>().expect("Unable to parse chunk2 offset to usize");
          modified.push(offset);

        } else {
          if cfg.verbosity > 3 {
            println!(": unchanged: {}: ({})",chunk1_id,chunk1_payload[0]);
          }
        }
        chunk1_todelete.push(chunk1_payload[0].to_string());
        chunk2_todelete.push(chunk1_payload[0].to_string());
      },
      None => {
        if cfg.verbosity > 0 {
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

/// `build_chunkindex_from_xml` Parses the XMLs and builds chunkindexes out of them.
pub fn build_chunkindex_from_xml(cfg: &Config) -> Result<(),String> {
  let mut chunk_index1:ChunkIndex;
  let mut chunk_index2:ChunkIndex;
  match read_file_in_chunks(cfg,&cfg.input_filename1) {
    Ok((chunk_index, num_records)) => {
      chunk_index1 = chunk_index;
      println!("Consumed {} records", num_records);
    },
    Err(err) => {
      return Err(format!("Error processing file: {}",err));
    }
  }
  match read_file_in_chunks(cfg,&cfg.input_filename2) {
    Ok((chunk_index, num_records)) => {
      chunk_index2 = chunk_index;
      println!("Consumed {} records", num_records);
    },
    Err(err) => {
      return Err(format!("Error processing file: {}",err));
    }
  }
  calculate_diff(&cfg, &mut chunk_index1, &mut chunk_index2);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::{Bencher,black_box};

  #[test]
  fn it_gets_id() {
    let verbosity = 1i8;
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
      verbosity,
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_3>A</KEY_3><KEY_2></KEY_2><KEY_1>1</KEY_1>".to_owned();
    let fake_id = "TESTUNIT".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&fake_id,verbosity),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "
    
    <KEY_1>1</KEY_1>
      <KEY_3>A</KEY_3>
      <KEY_2></KEY_2>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&fake_id,verbosity),true);
    assert_eq!(chunk_id.join("&"),"1&&A".to_owned());
    let test_xml = "NotAnXML".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&fake_id,verbosity),false);
    let test_xml = "<KEY_3>A</KEY_3>".to_owned();
    assert_eq!(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&fake_id,verbosity),false);
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      "MEMORY".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
      verbosity,
    );
    let mut html_chunk_id: Vec<String> = vec![
      String::with_capacity(64),
    ];
    let test_html = "<html><head></head> <body> <div><li>1</li></div> </body> </html>".to_owned();
    assert_eq!(get_id(&test_html,&cfg.xml_keys,&mut html_chunk_id,&fake_id,verbosity),true);
    assert_eq!(html_chunk_id.join("&"),"1".to_owned());
  }
  #[test]
  fn it_gets_chunks() {
    let cfg = Config::new(
        "INVALID".to_owned(),
        "IMPORTANT_DATA".to_owned(),
        "MEMORY".to_owned(),
        "MEMORY".to_owned(),
        50usize,
        "test".to_owned(),
        10i8,
        "tcp://*:5555".to_owned(),
        0i8,
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
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
      0i8,
    );
    let mut test_html = "<html><head></head> <body> <div><li>1</li></div> <div><li>2</li></div> <div><li>3</li></div> </body> </html>".to_owned();
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>1</li>".to_owned(),32usize)));
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>2</li>".to_owned(),6usize)));
    assert_eq!(get_xml_chunk(&mut test_html,&cfg),Some(("<li>3</li>".to_owned(),6usize)));
  }
  #[test]
  fn it_adds_chunks() {
    let file1 = "tests/test1-dup.xml".to_owned();
    let file2 = "tests/test1-dup.xml".to_owned();
    let cfg = Config::new(
      "li".to_owned(),
      "div".to_owned(),
      file1,
      file2,
      10usize,
      "checksum".to_owned(),
      1i8, // The assert has a fixed expected offset, so we need to do this serially.
      "tcp://*:5671".to_owned(),
      0i8,
    );
    // There are 2 duplicate IDs in chunks in this file. in the same line.
    // <div><li>1</li></div><div><li>1</li></div>
    // 01234^67890123456789012345^7890
    let duplicate_id_tag = "<li>1</li>".to_owned();
    let id_sha = calculate_checksum(&duplicate_id_tag);
    match read_file_in_chunks(&cfg, &cfg.input_filename1) {
      Err(err) => assert_eq!(format!("At offset {}, found existing key {} at sha&offset {}&{}",26usize,1,id_sha,5usize),err),
      Ok((_,_))  => assert_eq!(0,1),
    };
  }
  #[bench]
  fn bench_get_id(b: &mut Bencher) {
    let verbosity = 1i8;
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
      verbosity,
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&"BENCHUNIT".to_owned(),verbosity)
    });
   }
  #[bench]
  fn bench_get_id_blackbox(b: &mut Bencher) {
    let verbosity = 1i8;
    let cfg = Config::new(
      "KEY_1,KEY_2,KEY_3".to_owned(),
      "NOTHING".to_owned(),
      "MEMORY".to_owned(),
      "MEMORY".to_owned(),
      10usize,
      "checksum".to_owned(),
      10i8,
      "tcp://*:5555".to_owned(),
      verbosity,
    );
    let mut chunk_id: Vec<String> = vec![
      String::with_capacity(64),
      String::with_capacity(64),
      String::with_capacity(64),
    ];
    let test_xml = "<KEY_2></KEY_2><KEY_3>A</KEY_3><KEY_1>1</KEY_1>".to_owned();
    b.iter(|| {
      for _ in 1..1000 {
        black_box(get_id(&test_xml,&cfg.xml_keys,&mut chunk_id,&"BENCHUNIT".to_owned(),verbosity));
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
      "tests/test1.xml".to_owned(),
      10usize,
      "checksum".to_owned(),
      1i8,
      "tcp://*:5555".to_owned(),
      0i8,
    );
    b.iter(|| {
      for _ in 1..1000 {
        black_box(read_file_in_chunks(&cfg,&cfg.input_filename1).unwrap());
      }
    });
  }
}
