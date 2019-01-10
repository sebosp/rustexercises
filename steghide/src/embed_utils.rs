
#[derive(Builder, Default)]
pub struct EmbedUtils{
    enc_algo: super::encryption_algorithm::EncryptionAlgorithm,
    enc_mode: super::encryption_mode::EncryptionMode,
    compression_level: u8,
    nochecksum: bool,
    embed_name: bool,
    embedfile: super::OptionalFile,
    passphrase: String,
    filename: String,
}
impl EmbedUtils {
    pub fn get_bit_string(self) -> super::bit_string::BitString {
        unimplemented!("BitString not Implemented.")
    }
}
