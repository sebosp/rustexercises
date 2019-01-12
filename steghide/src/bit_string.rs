#[derive(Default, Builder, Clone)]
pub struct BitString {
    /// the number of bits in Data
    length: u32,
    /// the arity that will be used for getLength/getNAry/appendNAry
    /// Arity is also seen as unsigned char which seems to be u8
    arity: u8,
    /// the number of Bits per n-ary digit (where n is Arity)
    arity_nbits: u16,
    /// the actual data
    data: Vec<u8>,
}
impl BitString{
    pub fn new() -> BitString {
        // XXX: Could is overloaded many times
        BitString {
            length: 0,
            arity: 0,
            arity_nbits: 0,
            data: vec![]
        }
    }
}