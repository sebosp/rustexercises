#[derive(Copy, Clone, PartialEq)]
pub enum EncryptionMode {
    ECB,
    CBC,
    OFB,
    CFB,
    NOFB,
    NCFB,
    CTR,
    Stream
}

impl Default for EncryptionMode {
    fn default() -> EncryptionMode { EncryptionMode::ECB }
}