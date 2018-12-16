//! Steghide Crate
//! Allows extraction or embedding of data inside an image,
//! A passphrase is needed for either extraction or embedding.
//!

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate rpassword;

pub mod cli;
pub mod encryption_algorithm;
pub mod encryption_mode;
pub mod embedder;

/// `CommandMode` defines methods of operations of the library
#[derive(PartialEq)]
pub enum CommandMode {
    Embed,
    Extract,
    Info,
    Encinfo,
    PrintFreqs,
}

#[derive(PartialEq)]
pub enum RequestMode {
    CommandLine,
    HTTPRequest,
}
/// `DebugMode` defines different way to show debug information on the operations
#[derive(PartialEq,Debug)]
pub enum DebugMode {
    PrintGraph,
    PrintGmlGraph,
    PrintGmlVertex(u64,u64), // RecDepth, StartVertex
    PrintStats,
    DebugLevel(u64),
    Check,
} 

/// `StegHideRequest` contains a request for the library to operate on
#[derive(PartialEq)]
pub struct StegHideRequest{
    passphrase: String,
    compression_level: u8,
    command: CommandMode,
    debug_mode: Option<DebugMode>,
    embedfile: OptionalFile,
    extractfile: OptionalFile,
    coverfile: OptionalFile,
    stegofile: OptionalFile,
    marker: String,
    nochecksum: bool,
    embed_name: bool,
    enc_algo: encryption_algorithm::EncryptionAlgorithm,
    enc_mode: encryption_mode::EncryptionMode,
    radius: u64,
    goal: f64,
    force: bool,
    verbosity: i8,
    check: bool,
    file_list: Vec<String>,
    request_mode: RequestMode,
}

/// `StegHideRequest` defines the main operations of the library
/// This is taken from Session.cc
impl StegHideRequest {
    pub fn run(self) -> Result<String, String>{
        if self.command == CommandMode::Embed {
            embedder::Embedder::new(&self)?.run()
        } else {
            Ok("non-embed is not implemented".to_string())
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum OptionalFile{
    None,
    Stdin,
    Some(String),
}
impl OptionalFile{
    fn is_none(&self) -> bool {
        *self == OptionalFile::None
    }
    fn is_some(&self) -> bool {
        match *self{
            OptionalFile::None => false,
            OptionalFile::Stdin => false,
            _ => true
        }
    }
    fn is_stdin(&self) -> bool {
        *self == OptionalFile::Stdin
    }
}