//! `Config`
//! contains the configuration parsed from command line.

extern crate getopts;
extern crate std;
use super::*;
use getopts::Options;
use std::path::Path;
use std::fmt;

#[derive(Default)]
pub struct Config {
  pub mode: String,
  pub input_filename1: String,
  pub input_filename2: String,
  pub xml_keys: Vec<String>,
  pub chunk_delimiter: String,
  pub chunk_size: usize,
  pub concurrency: i8,
  pub bind_address: String,
  pub verbosity: i8,
  pub use_index_files: bool,
}

impl Config {
  pub fn with_mode(mut self, mode: String) -> Config {
    self.mode = match mode.as_ref() {
      "help"|"checksum"|"validate"|"compare" => {
         mode
      }
      _ => {
        // Defaults to diff mode.
       "diff".to_string()
      }
    };
    self
  }
  pub fn with_file(mut self, input: String) -> Config {
    if input.len() == 0 {
      return self
    }
    if self.input_filename1 == "" {
      self.input_filename1 = input;
    } else if self.input_filename2 == "" {
      self.input_filename2 = input;
    }
    self
  }
  pub fn with_xml_keys(mut self, input: String) -> Config {
    if input.len() == 0 {
      return self;
    }
    let mut xml_keys:Vec<String> = vec![];
    for key in input.split(",") {
      // Prepend an / for the closing tag
      xml_keys.push(format!("/{}",key));
    }
    self.xml_keys = xml_keys;
    self
  }
  pub fn with_chunk_delimiter(mut self, input: String) -> Config {
    self.chunk_delimiter = input;
    self
  }
  pub fn with_chunk_size(mut self, input: usize) -> Config {
    self.chunk_size = input * 1024;
    self
  }
  pub fn with_concurrency(mut self, input: i8) -> Config {
    self.concurrency = input;
    self
  }
  pub fn with_bind_address(mut self, input: String) -> Config {
    if input.len() > 0 {
      self.bind_address = input;
    }
    self
  }
  pub fn with_verbosity(mut self, input: i8) -> Config {
    self.verbosity = input;
    self
  }
  pub fn with_use_index_files(mut self, input: bool) -> Config {
    self.use_index_files = input;
    self
  }
  pub fn build(mut self) -> Result<Config, String> {
    match self.mode.as_ref() {
      "help" => {
         Ok(self)
      },
      "checksum" => {
         Ok(self)
      },
      "validate" => {
         Ok(self)
      },
      "compare" => {
         Ok(self)
      },
      "diff" => {
         Ok(self)
      },
      "index" => {
        if self.chunk_delimiter == "" {
          return Err("Missing XML Chunk delimiter".to_string());
        }
        if self.xml_keys.len() == 0 {
          return Err("Missing xml key fields".to_string());
        }
        if ! Path::new(&self.input_filename1).exists() {
          return Err(format!("Input file '{}' does not exist.",self.input_filename1));
        }
        if ! Path::new(&self.input_filename2).exists() {
          return Err(format!("Input file '{}' does not exist.",self.input_filename2));
        }
        Ok(self)
      }
      _ => {
        // Defaults to diff mode.
       self.mode = "diff".to_string();
       Ok(self)
      }
    }
  }
  pub fn new() -> Config {
    // Defaults
    Config{
        xml_keys: vec![],
        chunk_delimiter: "".to_string(),
        input_filename1: "".to_string(),
        input_filename2: "".to_string(),
        chunk_size: 32usize * 1024,
        mode: "help".to_string(),
        concurrency: 4i8,
        bind_address: "ipc://test".to_string(),
        verbosity: 1i8,
        use_index_files: true
    }
  }
  pub fn from_getopts(args: std::env::Args) -> Result<Config, String> {
    let args: Vec<String> = args.collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("m", "mode", "Operating mode, see help for more info", "MODE");
    opts.optopt("i1", "inputFile1", "Input File 1", "FILE");
    opts.optopt("i2", "inputFile2", "Input File 2", "FILE");
    opts.optopt("o", "outputFile", "Output File ", "FILE");
    opts.optopt("d", "delimiter", "XML Chunk Delimiter", "GROUPINGTAG");
    opts.optopt("s", "size", "File read size", "SIZE");
    opts.optopt("b", "bindAddress", "Bind to this address", "bind socket");
    opts.optopt("k", "keyFields", "Comma separated list of tags in the XML Chunk that will be use to set a unique ID", "TAG1,TAG2,TAG3");
    opts.optopt("c", "concurrency", "Run the parsing in separate threads", "SIZE");
    opts.optopt("v", "verbosity", "Set verbosity level", "SIZE");
    opts.optopt("u", "use_index_files", "Use Index Files", "BOOL");
    let matches = match opts.parse(&args[..]) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
      return Ok(Config::new().with_mode("help".to_string()));
    }
    let mut res = Config::new()
      .with_mode(matches.opt_str("m").unwrap_or_default())
      .with_chunk_delimiter(matches.opt_str("d").unwrap_or_default())
      .with_xml_keys(matches.opt_str("k").unwrap_or_default())
      .with_file(matches.opt_str("i1").unwrap_or_default())
      .with_file(matches.opt_str("i2").unwrap_or_default())
      .with_chunk_delimiter(matches.opt_str("d").unwrap_or_default())
      .with_use_index_files(matches.opt_present("u"))
      .with_bind_address(matches.opt_str("b").unwrap_or_default());

    match matches.opt_str("s") {
      Some(chunk_size) => {
        match chunk_size.parse::<usize>() {
          Ok(val) => res = res.with_chunk_size(val),
          Err(err) => return Err(format!("Error: unable to parse chunk size (-s) '{}': {}", chunk_size, err)),
        };
      },
      None => {},
    }
    match matches.opt_str("c") {
      Some(concurrency) => {
        match concurrency.parse::<i8>() {
          Ok(val) => res = res.with_concurrency(val),
          Err(err) => return Err(format!("Error: unable to parse concurrency (-c) '{}': {}.", concurrency, err)),
        };
      },
      None => {},
    }
    match matches.opt_str("v") {
      Some(verbosity_level_string) => {
        match verbosity_level_string.parse::<i8>() {
          Ok(val) => res = res.with_verbosity(val),
          Err(err) => {
            eprintln!("Warning: unable to parse verbosity level (-v) value '{}': {}. Using default: {}", verbosity_level_string, err,res.verbosity);
          }
        }
      },
      None => {},
    }
    Ok(res)
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
    println!("-i1: inputFile1");
    println!("-i2: inputFile2");
    println!("-o: outputFile");
    println!("-s: Read file in these many bytes");
    println!("-d: delimiter: XML Chunk Delimiter");
    println!("-c: concurrency: Use N threads for parsing");
    println!("-v: verbosity: Use N level of verbosity");
    println!("-k: keyFields: Comma separated list of tags in the XML Chunk that will be use to set a unique ID");
  }
  /// run checks the current mode and acts on it.
  pub fn run(self) -> Result<(), String> {
    match self.mode.as_ref() {
      "help" => {
         self.print_usage();
         Ok(())
      },
      "checksum" => {
        match write_diff_files(&self){
          Ok(()) => {
            Ok(())
          },
          Err(err) => {
            Err(err.to_string())
          }
        }
      },
      "validate" => {
         self.print_usage();
         Ok(())
      }
      "compare" => {
         self.print_usage();
         Ok(())
      }
      "diff" => {
        if self.use_index_files {
          self.print_usage();
          Ok(())
        } else {
          Ok(())
        }
      }
      _ => {
        Err("Unknown operational mode".to_owned())
      }
    }
  }
}
impl fmt::Display for Config {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Config{{
      mode: {}
      input_filename1: {}
      input_filename2: {}
      xml_keys: {:?}
      chunk_delimiter: {}
      chunk_size: {}
      concurrency: {}
      bind_address: {}
      verbosity: {}
      use_index_files: {}
      }}\n",
      self.mode,
      self.input_filename1,
      self.input_filename2,
      self.xml_keys,
      self.chunk_delimiter,
      self.chunk_size,
      self.concurrency,
      self.bind_address,
      self.verbosity,
      self.use_index_files
      )
  }
}
