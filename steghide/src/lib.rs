//! Steghide Crate
//! Allows extraction or embedding of data inside an image,
//! A passphrase is needed for either extraction or embedding.
//!

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate stderrlog;

pub mod cli;
pub mod encryption_algorithm;
pub mod encryption_mode;

/// `CommandMode` defines methods of operations of the library
enum CommandMode {
    Embed,
    Extract,
    Info,
    Encinfo,
    PrintFreqs,
}

/// `DebugMode` defines different way to show debug information on the operations
enum DebugMode {
    PrintGraph,
    PrintGmlGraph,
    PrintGmlVertex(u64,u64), // RecDepth, StartVertex
    PrintStats,
    DebugLevel(u64),
    Check,
} 

/// `StegHideSetup` contains a request for the library to operate on
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