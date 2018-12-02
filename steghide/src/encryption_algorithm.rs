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
    pub fn from_string(cipher: String) -> Result<EncryptionAlgorithm,String> {
        let algo_cipher = match cipher.as_ref() {
            "Twofish" => Some(EncryptionAlgorithmCipher::Twofish),
            "Rijndael128" => Some(EncryptionAlgorithmCipher::Rijndael128),
            "Rijndael192" => Some(EncryptionAlgorithmCipher::Rijndael192),
            "Rijndael256" => Some(EncryptionAlgorithmCipher::Rijndael256),
            "Saferplus" => Some(EncryptionAlgorithmCipher::Saferplus),
            "Rc2" => Some(EncryptionAlgorithmCipher::Rc2),
            "Xtea" => Some(EncryptionAlgorithmCipher::Xtea),
            "Serpent" => Some(EncryptionAlgorithmCipher::Serpent),
            "Safersk64" => Some(EncryptionAlgorithmCipher::Safersk64),
            "Safersk128" => Some(EncryptionAlgorithmCipher::Safersk128),
            "Cast256" => Some(EncryptionAlgorithmCipher::Cast256),
            "Loki97" => Some(EncryptionAlgorithmCipher::Loki97),
            "Gost" => Some(EncryptionAlgorithmCipher::Gost),
            "Threeway" => Some(EncryptionAlgorithmCipher::Threeway),
            "Cast128" => Some(EncryptionAlgorithmCipher::Cast128),
            "Blowfish" => Some(EncryptionAlgorithmCipher::Blowfish),
            "Des" => Some(EncryptionAlgorithmCipher::Des),
            "Tripledes" => Some(EncryptionAlgorithmCipher::Tripledes),
            "Enigma" => Some(EncryptionAlgorithmCipher::Enigma),
            "Arcfour" => Some(EncryptionAlgorithmCipher::Arcfour),
            "Panama" => Some(EncryptionAlgorithmCipher::Panama),
            "Wake" => Some(EncryptionAlgorithmCipher::Wake),
            _ => None,
        };
        if algo_cipher.is_none() {
            Err("Unknown cipher".to_string())
        } else {
            Ok(EncryptionAlgorithm {
                cipher: algo_cipher,
            })
        }
    }
}
