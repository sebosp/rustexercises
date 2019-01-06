use super::OptionalFile;
use super::cli::StegHideCommandBuilder;
use std::path::Path;
use std::fs::File;
use std::io::{self, Read};

#[derive(PartialEq, Debug)]
enum OpenMode {
    Read,
    Write
}
#[derive(Debug)]
pub struct BinaryIO {
    mode: OpenMode,
    file: OptionalFile,
    data: Vec<u8>,
    handle: Option<File>,
}
impl BinaryIO {
    pub fn new(file: &OptionalFile, open_mode: &'static str) -> Result<BinaryIO,String> {
        let open_mode = match open_mode.as_ref() {
            "read"|"r"|"READ" => OpenMode::Read,
            "write"|"w"|"WRITE" => OpenMode::Write,
            _ => {
                error!("Unknown read mode: {}", open_mode);
                return Err(format!("BinaryIO: Unknown read mode: {}", open_mode));
            }
        };
        let file_handle = match file {
            OptionalFile::None => {
                error!("Missing filename in constructor");
                return Err("BinaryIO: Missing filename in constructor.".to_string());
            },
            OptionalFile::Stdin => {
                if open_mode == OpenMode::Write {
                    error!("OpenMode Write is not possible for Stdin");
                    return Err("BinaryIO: OpenMode Write is not possible for Stdin".to_string());
                }
                None
            },
            OptionalFile::Stdout => {
                if open_mode == OpenMode::Read {
                    error!("OpenMode Read is not possible for Stdout");
                    return Err("BinaryIO: OpenMode Read is not possible for Stdout".to_string());
                }
                None
            },
            OptionalFile::Some(filename) => {
                let file_open = File::open(&filename);
                match file_open {
                    Ok(f) => Some(f),
                    Err(err) => return Err(err.to_string()),
                }
            }
        };
        Ok(BinaryIO {
            mode: open_mode,
            file: file.clone(),
            data: vec![],
            handle: file_handle,
        })
    }
    pub fn check_force(self, request: super::StegHideRequest) -> bool {
        if !request.force {
            match self.file{
                OptionalFile::Some(fname) => {
                    if Path::new(&fname).exists(){
                        match request.request_mode {
                            super::RequestMode::HTTPRequest => false,
                            super::RequestMode::CommandLine => {
                                let user_question = format!("the file \"{}\" already exist. overwrite ?",fname);
                                StegHideCommandBuilder::request_user_bool_response(user_question)
                            }
                        }
                    } else {
                        true
                    }
                },
                OptionalFile::None => true,
                OptionalFile::Stdin => true,
                OptionalFile::Stdout => true,
            }
        } else {
            return true
        }
    }
    pub fn read(self) -> Option<Vec<u8>> {
        match self.file {
            OptionalFile::Stdin => {
                info!("reading data from standard input...");
                let mut buffer = Vec::new();
                io::stdin().read_to_end(&mut buffer).unwrap();
                Some(buffer)
            },
            OptionalFile::Some(filename) => {
                info!("reading data from file {}",filename);
                let mut buffer = Vec::new();
                let mut f = File::open(filename).unwrap();
                f.read_to_end(&mut buffer).unwrap();
                Some(buffer)
            },
            OptionalFile::Stdout => {
                // The constructor won't let us get this far tho.
                error!("Attempting to read on Stdout");
                None
            },
            OptionalFile::None => {
                // The constructor won't let us get this far tho.
                error!("Attempting to read on None");
                None
            }
        }
    }
}
impl PartialEq for BinaryIO {
    // `eq` Compares two BinaryIO structs.
    // The File Handle is ignored since it probably points to an fd
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode &&
        self.file == other.file &&
        self.data == other.data
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_checks_open_mode_types() {
        let expected_res = BinaryIO {
            mode: OpenMode::Read,
            file: OptionalFile::Stdin,
            data: vec![],
            handle: None,
        };
        let missing_file_test = super::BinaryIO::new(&OptionalFile::None, "r");
        assert_eq!(missing_file_test, Err("BinaryIO: Missing filename in constructor.".to_string()));
        let read_stdin_test = super::BinaryIO::new(&OptionalFile::Stdin, "r");
        assert_eq!(read_stdin_test, Ok(expected_res));
        let read_stdout_test = super::BinaryIO::new(&OptionalFile::Stdout, "r");
        assert_eq!(read_stdout_test, Err("BinaryIO: OpenMode Read is not possible for Stdout".to_string()));
        // Test Write
        let expected_res = BinaryIO {
            mode: OpenMode::Write,
            file: OptionalFile::Stdin,
            data: vec![],
            handle: None,
        };
        let write_stdin_test = super::BinaryIO::new(&OptionalFile::Stdin, "w");
        assert_eq!(write_stdin_test, Err("BinaryIO: OpenMode Write is not possible for Stdin".to_string()));
        let write_stdout_test = super::BinaryIO::new(&OptionalFile::Stdout, "w");
        assert_eq!(read_stdin_test, Ok(expected_res));
    }
}