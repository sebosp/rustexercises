#[derive(PartialEq)]
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
#[derive(PartialEq)]
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
            "twofish" => Some(EncryptionAlgorithmCipher::Twofish),
            "rijndael-128" => Some(EncryptionAlgorithmCipher::Rijndael128),
            "rijndael-192" => Some(EncryptionAlgorithmCipher::Rijndael192),
            "rijndael-256" => Some(EncryptionAlgorithmCipher::Rijndael256),
            "saferplus" => Some(EncryptionAlgorithmCipher::Saferplus),
            "rc2" => Some(EncryptionAlgorithmCipher::Rc2),
            "xtea" => Some(EncryptionAlgorithmCipher::Xtea),
            "serpent" => Some(EncryptionAlgorithmCipher::Serpent),
            "safer-sk64" => Some(EncryptionAlgorithmCipher::Safersk64),
            "safer-sk128" => Some(EncryptionAlgorithmCipher::Safersk128),
            "cast-256" => Some(EncryptionAlgorithmCipher::Cast256),
            "loki97" => Some(EncryptionAlgorithmCipher::Loki97),
            "gost" => Some(EncryptionAlgorithmCipher::Gost),
            "threeway" => Some(EncryptionAlgorithmCipher::Threeway),
            "cast-128" => Some(EncryptionAlgorithmCipher::Cast128),
            "blowfish" => Some(EncryptionAlgorithmCipher::Blowfish),
            "des" => Some(EncryptionAlgorithmCipher::Des),
            "tripledes" => Some(EncryptionAlgorithmCipher::Tripledes),
            "enigma" => Some(EncryptionAlgorithmCipher::Enigma),
            "arcfour" => Some(EncryptionAlgorithmCipher::Arcfour),
            "panama" => Some(EncryptionAlgorithmCipher::Panama),
            "wake" => Some(EncryptionAlgorithmCipher::Wake),
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
