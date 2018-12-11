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

/// `CommandMode` defines methods of operations of the library
#[derive(PartialEq)]
pub enum CommandMode {
    Embed,
    Extract,
    Info,
    Encinfo,
    PrintFreqs,
}

/// `DebugMode` defines different way to show debug information on the operations
#[derive(PartialEq)]
pub enum DebugMode {
    PrintGraph,
    PrintGmlGraph,
    PrintGmlVertex(u64,u64), // RecDepth, StartVertex
    PrintStats,
    DebugLevel(u64),
    Check,
} 

/// `StegHideSetup` contains a request for the library to operate on
#[derive(PartialEq)]
pub struct StegHideSetup{
    passphrase: String,
    compression_level: u8,
    command: Option<CommandMode>,
    debug: Option<DebugMode>
}

/// `StegHideSetup` defines the main operations of the library
impl StegHideSetup {
    pub fn run(self) -> Result<String, String>{
        Ok(String::from("Finished running"))
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