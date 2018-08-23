//! `Config`
//! contains the configuration parsed from command line.

extern crate getopts;
extern crate std;
use getopts::Options;
use std::path::Path;

pub struct Config {
  pub mode: String,
  pub input_filename: String,
  pub xml_keys: Vec<String>,
  pub chunk_delimiter: String,
  pub chunk_size: usize,
  pub concurrency: i8,
  pub bind_address: String,
}

impl Config {
  pub fn new(keys: String,chunk_delimiter: String, input_filename: String, chunk_size: usize, mode: String, concurrency: i8, bind_address: String) -> Config {
    let mut xml_keys:Vec<String> = vec![];
    for key in keys.split(",") {
      // Prepend an / for the closing tag
      xml_keys.push(format!("/{}",key));
    }
    Config{
        xml_keys: xml_keys,
        chunk_delimiter: chunk_delimiter,
        input_filename: input_filename,
        chunk_size: chunk_size * 1024,
        mode: mode,
        concurrency: concurrency,
        bind_address: bind_address,
    }
  }
  pub fn from_getopts(args: std::env::Args) -> Result<Config, &'static str> {
    let args: Vec<String> = args.collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("m", "mode", "Operating mode, see help for more info", "MODE");
    opts.optopt("i", "inputFile", "Input File ", "FILE");
    opts.optopt("o", "outputFile", "Output File ", "FILE");
    opts.optopt("d", "delimiter", "XML Chunk Delimiter", "GROUPINGTAG");
    opts.optopt("s", "size", "File read size", "SIZE");
    opts.optopt("b", "bindAddress", "Bind to this address", "bind socket");
    opts.optopt("k", "keyFields", "Comma separated list of tags in the XML Chunk that will be use to set a unique ID", "TAG1,TAG2,TAG3");
    opts.optopt("c", "concurrency", "Run the parsing in separate threads", "SIZE");
    let matches = match opts.parse(&args[..]) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
    };

    let mut mode = "help".to_owned(); // Default val
    let mut xml_keystring = String::new();
    let mut bind_address = String::new();
    let mut chunk_delimiter = String::new();
    let mut input_filename = String::new();
    let mut chunk_size = 0usize;
    let mut concurrency = 1i8;

    if matches.opt_present("h") {
      return Ok(Config::new(xml_keystring,chunk_delimiter,input_filename,chunk_size,mode,concurrency,bind_address));
    }
    match matches.opt_str("m") {
      Some(opt_mode) => {
        mode = opt_mode.to_owned();
      },
      None => return Err("Missing -m mode parameter"),
    }
    match matches.opt_str("d") {
      Some(delimiter) => chunk_delimiter = delimiter,
      None => return Err("Missing XML Chunk delimiter"),
    }
    match matches.opt_str("m") {
      Some(keys) => xml_keystring = keys,
      None => return Err("Missing xml key fields"),
    }
    match matches.opt_str("i") {
      Some(file) => {
        if Path::new(&file).exists() {
          input_filename = file
        } else {
          return Err("Input file does not exist.");
        }
      },
      None => return Err("Missing -i input file parameter"),
    };
    chunk_size = match matches.opt_str("s") {
      Some(size) => size.parse::<usize>().unwrap(),
      None => 128usize,
    };
    concurrency = match matches.opt_str("c") {
      Some(size) => size.parse::<i8>().unwrap(),
      None => 10i8,
    };
    match matches.opt_str("b") {
      Some(address) => {
        bind_address = address.to_owned();
      },
      None => bind_address = "tcp://*:5555".to_owned(),
    }
    Ok(Config::new(xml_keystring,chunk_delimiter,input_filename,chunk_size,mode,concurrency,bind_address))
  }
  /// `print_usage` prints program GetOpt usage.
  pub fn print_usage(self) {
    println!("Usage: -m MODE -i FILE -k TAGID1,TAGID2,TAGID3 -d CHUNKDELIMITER [options]");
    println!("-h: help print this help menu");
    println!("-b: bind address");
    println!("-m: operation mode:");
    println!("    - checksum: Creates the chunk indexes of a file");
    println!("    - validate: Validates the chunk indexes file integrity");
    println!("    - compare: Compares two chunk indexes files");
    println!("    - checksum: Calculates the chunk checksums of a file");
    println!("    - diff: Displays the difference of two XML files");
    println!("-i: inputFile");
    println!("-o: outputFile");
    println!("-s: Read file in these many bytes");
    println!("-d: delimiter: XML Chunk Delimiter");
    println!("-c: concurrency: Use N threads for parsing");
    println!("-k: keyFields: Comma separated list of tags in the XML Chunk that will be use to set a unique ID");
  }
  /// Returns the number of records or the error message
  pub fn run(self) -> Result<i64, String> {
    match self.mode.as_ref() {
      "help" => {
         self.print_usage();
         Ok(0)
      },
      "checksum" => {
         self.print_usage();
         Ok(0)
      }
      "validate" => {
         self.print_usage();
         Ok(0)
      }
      "compare" => {
         self.print_usage();
         Ok(0)
      }
      "diff" => {
         self.print_usage();
         Ok(0)
      }
      _ => {
        Err("Unknown operational mode".to_owned())
      }
    }
  }
}
