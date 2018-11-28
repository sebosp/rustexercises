enum EncryptionAlgorithmCipher {
    Twofish,
    Rijndael128,
    Rijndael192,
    Rijndael256,
    Saferplus,
    Rc2,
    Xtea,
    Serpent,
    Safersk64,
    Safersk128,
    Cast256,
    Loki97,
    Gost,
    Threeway,
    Cast128,
    Blowfish,
    Des,
    Tripledes,
    Enigma,
    Arcfour,
    Panama,
    Wake,
}
pub struct EncryptionAlgorithm{
    cipher: Option<EncryptionAlgorithmCipher>
}

impl EncryptionAlgorithm{
    pub fn new() -> EncryptionAlgorithm {
        EncryptionAlgorithm {
            cipher: None,
        }
    }

}
