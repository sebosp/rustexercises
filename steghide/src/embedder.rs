use super::binary_io::BinaryIO;
/// `Embedder` runs the Embedding operations
pub struct Embedder{
    embed_file_contents: Option<Vec<u8>>,
    enc_algo: super::encryption_algorithm::EncryptionAlgorithm,
    enc_mode: super::encryption_mode::EncryptionMode,
    compression_level: u8,
    nochecksum: bool,
    embed_name: bool,
    embedfile: super::OptionalFile,
}

impl Embedder{
    pub fn new(request: &super::StegHideRequest) -> Result<Embedder,String> {
        let buffer = BinaryIO::new(&request.embedfile, "read")?
            .read();
        Ok(Embedder{
            embed_file_contents: buffer,
            enc_algo: request.enc_algo,
            enc_mode: request.enc_mode,
            compression_level: request.compression_level,
            nochecksum: request.nochecksum,
            embed_name: request.embed_name,
            embedfile: request.embedfile,
        })
    }
    pub fn run(self) -> Result<String, String> {
        // create bitstring to be embedded
        let mut embed_filename:String; 
        if self.embed_name {
            embed_filename = match self.embedfile {
                super::OptionalFile::Some(fname) => fname.clone(),
                _ => "".to_string(),
            };
        }
        let embed_data = super::EmbedUtilsBuilder::new(
                super::EmbedUtils::Mode::Embed,
                self.request.Passphrase,
            );

        Ok(String::from("embedder::run() is not implemented"))
    }
}