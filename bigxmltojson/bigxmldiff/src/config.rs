//! `Config`
//! contains the configuration parsed from command line.

extern crate getopts;
use getopts::Options;
use std::process;
use std::io::prelude::*;
use std::env;

pub struct Config {
  mode: String,
  input_filename: String,
  xml_keys: Vec<String>,
  chunk_delimiter: String,
}

impl Config {
  /// `getopt_print_usage` prints program GetOpt usage.
  pub fn getopt_print_usage(opts: Options) {
    let brief = format!("Usage: MODE FILE TAGID1,TAGID2,TAGID3 CHUNKDELIMITER [options]");
    print!("{}", opts.usage(&brief));
  }
  pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
    let args: Vec<String> = args.collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("m", "mode", "Calculates the checksum of a given file", "MODE");
    opts.optopt("i", "inputFile", "Input File ", "FILE");
    opts.optopt("o", "outputFile", "Output File ", "FILE");
    opts.optopt("d", "delimiter", "XML Chunk Delimiter", "ITEM");
    opts.optopt("k", "keyFields", "Comma separated list of tags in the XML Chunk that will be use to set a unique ID", "TAG1,TAG2,TAG3");
    let matches = match opts.parse(&args[..]) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
    };
    opts.optflag("h", "help", "print this help menu");
    if matches.opt_present("h") {
        print_usage(&program, opts);
        mode = "usage".to_owned();
    }
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
    };
    Ok(Config{
      xml_keys: matches.opt_str("k").split(",").collect(),
      chunk_delimiter: matches.opt_str("d").clone(),
      input_filename: matches.opt_str("i").clone(),
      mode: matches.opt_str("m").clone(),
    })
  }
  /// Returns the number of records or the error message
  pub fn run(self) -> Result<i64, String>{
    if (self.mode == "help"){
      getopt_print_usage();
      Ok(0)
    }
  }
}
