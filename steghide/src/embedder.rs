use super::binary_io::BinaryIO;
use super::embed_utils::*;
/// `Embedder` runs the Embedding operations
pub struct Embedder{
    embed_file_contents: Option<Vec<u8>>,
    utils: EmbedUtils,
}

impl Embedder{
    pub fn new(request: &super::StegHideRequest) -> Result<Embedder,String> {
        let buffer = BinaryIO::new(&request.embedfile, "read")?
            .read();
        let mut embed_filename = "".to_string();
        if request.embed_name {
            embed_filename = match &request.embedfile {
                super::OptionalFile::Some(fname) => fname.clone(),
                _ => "".to_string(),
            };
        }
        let utils = super::embed_utils::EmbedUtilsBuilder::default()
            .passphrase(request.passphrase.clone())
            .filename(embed_filename.clone())
            .enc_algo(request.enc_algo.clone())
            .enc_mode(request.enc_mode)
            .compression_level(request.compression_level)
            .nochecksum(request.nochecksum)
            .embed_name(request.embed_name)
            .embedfile(request.embedfile.clone())
            .build()?;
        Ok(Embedder{
            embed_file_contents: buffer,
            utils: utils,
        })
    }
    pub fn run(self) -> Result<String, String> {
        // create bitstring to be embedded
	    self.utils.get_bit_string() ;
        Ok(String::from("embedder::run() is not implemented"))
    }
}