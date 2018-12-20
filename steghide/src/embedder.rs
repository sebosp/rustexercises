use super::OptionalFile;
use std::fs::File;
use std::io::Read;
use std::io::{self, Read};
use std::io::prelude::*;
pub struct Embedder{
    embed_file_contents: Vec<u8>,
}

impl Embedder{
    pub fn new(request: &super::StegHideRequest) -> Result<Embedder,String> {
        let mut buffer = Vec::new();
        match &request.embedfile {
            OptionalFile::None => {
                error!("Embedder new() needs a data source to work in");
                return Err("Missing Embed file".to_string());
            },
            OptionalFile::Stdin => {
                info!("reading secret data from standard input...");
                io::stdin().read_to_string(&mut buffer).unwrap(); // XXX: Remove unwrap, move to binary_io
            },
            OptionalFile::Some(filename) => {
                info!("reading secret file {}",filename);
                let mut f = File::open(filename).unwrap(); // XXX: Remove unwrap, move to binary_io
                f.read_to_end(&mut buffer).unwrap();       // XXX: Remove unwrap, move to binary_io
            }
        };
        Ok(Embedder{
            embed_file_contents: buffer
        })
        // - Create vec of bytes
        // - Use binary_io read the bytes of the file/stdin into this vec
    }
    pub fn run(self) -> Result<String, String> {
        Ok(String::from("embedder::new() is not implemented"))
    }
}