use super::binary_io::BinaryIO;
use super::embed_utils::*;
/// `Embedder` runs the Embedding operations
pub struct Embedder{
    embed_file_contents: Option<Vec<u8>>,
    coverfile: super::OptionalFile,
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
            .with_passphrase(request.passphrase.clone())
            .with_filename(embed_filename.clone())
            .with_enc_algo(request.enc_algo.clone())
            .with_enc_mode(request.enc_mode)
            .with_compression_level(request.compression_level)
            .with_nochecksum(request.nochecksum)
            .with_embed_name(request.embed_name)
            .with_embedfile(request.embedfile.clone())
            .build()?;
        Ok(Embedder{
            embed_file_contents: buffer,
            coverfile: request.coverfile.clone(),
            utils: utils,
        })
    }
    pub fn run(self) -> Result<String, String> {
        // create bitstring to be embedded
	    let _to_embed = self.utils.get_bit_string();
        match self.coverfile {
            super::OptionalFile::Stdin => info!("Reading cover file from standard input..."),
            _ => info!("Unknown Operation Mode"),
        };

        Ok(String::from("embedder::run() is not implemented"))
    }
}