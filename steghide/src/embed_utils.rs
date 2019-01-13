
use super::bit_string::BitString;
use super::encryption_algorithm::EncryptionAlgorithm;

/// `EmbedUtilsOperation` allows the object to be used in different ways.
#[derive(Clone, PartialEq)]
pub enum EmbedUtilsOperation {
    Embed,
    Extract,
}
#[derive(Clone, PartialEq)]
pub enum EmbedUtilsState {
    ReadMagic,
    ReadVersion,
    ReadEncInfo,
    ReadNPlainBits,
    ReadEncrypted,
    End,
}
impl Default for EmbedUtilsOperation {
    fn default() -> EmbedUtilsOperation { EmbedUtilsOperation::Extract}
}

impl Default for EmbedUtilsState {
    fn default() -> EmbedUtilsState { EmbedUtilsState::ReadMagic}
}

#[derive(Builder, Default)]
#[builder(setter(prefix = "with"))]
pub struct EmbedUtils{
    enc_algo: super::encryption_algorithm::EncryptionAlgorithm,
    enc_mode: super::encryption_mode::EncryptionMode,
    compression_level: u8,
    nochecksum: bool,
    embed_name: bool,
    embedfile: super::OptionalFile,
    passphrase: String,
    filename: String,
    utils_operation: EmbedUtilsOperation,
    state: EmbedUtilsState,
    /// the minimum size of the part of the generatred BitString that is not the data
    #[builder(default = "50")]
    min_stego_header_size: u32,
    /// number of bits used to code the number of plain bits
    #[builder(default = "32")]
    nbits_nplain_bits: u32,
    /// number of bits used to code the number of uncompressed bits
    #[builder(default = "32")]
    nbits_nuncompressed_bits: u32,
    /// size of a crc32 checksum in bits
    #[builder(default = "32")]
    nbits_crc32: u32,
    /// version of this steghide embedding (stego compatibility of EmbData)
    #[builder(default = "0")]
    code_version: u8,
    /// steghide magic to recognize embedded data (the string "shm", 0x73688DUL)
    #[builder(default = "7563405")]
    steghide_magic: u32, // XXX: Is this really "shm"?
    /// steghide_magic size in bits
    #[builder(default = "24")]
    nbits_magic: u8,
    /// temporary storage of nbits read
    nplain_bits: u32,
    /// the number of bits that the caller must at least supply to addBits
    num_bits_requested: u32,
    /// exactly the number of bits that the next step will consume
    /// from Reservoir and addBits together
    num_bits_needed: u16,
    /// The Reservoir
    reservoir: super::bit_string::BitString,
    /// version read from input bitstring
    version_read: u8,
    /// The checksum
    crc32: u32,
    /// contains the actual message to be embedded
    data: Vec<u8>,

}
impl EmbedUtils {
    // pub fn new() -> EmbedUtils {
    // }
    pub fn init(&mut self) {
        self.code_version = 0;
        if self.utils_operation == EmbedUtilsOperation::Extract {
            self.num_bits_needed = self.nbits_magic as u16;
            self.num_bits_requested = self.nbits_magic as u32;
            self.version_read = self.code_version; // XXX: Why? EmbData.cc:35
            self.state = EmbedUtilsState::ReadMagic;
            self.reservoir = super::bit_string::BitString::new();
        }
    }
    pub fn get_bit_string(self) -> super::bit_string::BitString {
        unimplemented!("BitString is not implemented.")
    }
    pub fn strip_dir(input: String) -> String {
        unimplemented!("strip_dir is not implemented.")
    }
}