#[derive(Clone)]
pub struct BitString {
    /// the number of bits in Data
    length: u32,
    /// the arity that will be used for getLength/getNAry/appendNAry
    /// Arity is also seen as unsigned char which seems to be u8
    arity: u8, // Do setter skip and default work together?
    /// the number of Bits per n-ary digit (where n is Arity)
    arity_nbits: u16, // setArity requires some more logic
    /// the actual data
    data: Vec<u8>,
}
pub struct BitStringBuilder {
    length: Option<u32>,
    arity: Option<u8>,
    arity_nbits: Option<u16>,
    data: Option<Vec<u8>>,
}
impl BitString{
    pub fn new() -> BitString {
        // XXX: New is overloaded many times
        BitString {
            length: 0,
            arity: 0,
            arity_nbits: 0,
            data: vec![]
        }
    }
    pub fn append(&mut self, bit: bool) {
        if self.length % 8 == 0 { 
                self.data.push(0);
        }
        self.data[BitString::byte_pos(self.length)] |= (bit as u8) << BitString::bit_pos(self.length);
        self.length += 1;
    }
    pub fn bit_pos(n: u32) -> u8 {
        (n % 8) as u8
    }
    pub fn byte_pos(n: u32) -> usize {
        (n / 8) as usize
    }
}

impl Default for BitStringBuilder {
    fn default() -> Self {
        BitStringBuilder{
            length: Some(0),
            arity: Some(2),
            arity_nbits: None,
            data: None,
        }
    }
}
impl BitStringBuilder {
    #[doc = " the number of bits in Data"]
    #[allow(unused_mut)]
    pub fn with_length(&mut self, value: u32) -> &mut Self {
        let mut new = self;
        new.length = Some(value);
        new
    }
    #[doc =
            " the arity that will be used for getLength/getNAry/appendNAry"]
    #[doc = " Arity is also seen as unsigned char which seems to be u8"]
    pub fn with_arity(&mut self, value: u8) -> &mut Self {
        let mut new = self;
        new.arity = Some(value);
        let tmp = value;
        let arity_nbits = 0u16;
        while tmp > 1 {
            if tmp % 2 == 0 {
                error!("Only implemented for arity = 2^i");
                std::process::exit(1);
            }
            tmp /= 2;
            arity_nbits += 1;
        }
        self.arity_nbits = Some(arity_nbits);
        new
    }
    #[doc = " the number of Bits per n-ary digit (where n is Arity)"]
    #[allow(unused_mut)]
    pub fn with_arity_nbits(&mut self, value: u16) -> &mut Self {
        let mut new = self;
        new.arity_nbits = Some(value);
        new
    }
    #[doc = " the actual data"]
    #[allow(unused_mut)]
    pub fn with_data(&mut self, value: Vec<u8>) -> &mut Self {
        let mut new = self;
        new.data = Some(value);
        new
    }
    #[doc =
            "Builds a new `BitString`.\n\n# Errors\n\nIf a required field has not been initialized.\n"]
    pub fn build(&self) -> Result<BitString, String> {
        Ok(BitString{
            length:
                match self.length {
                    Some(ref value) =>
                    Clone::clone(value),
                    None => { 0 }
                },
            arity:
                match self.arity {
                    Some(ref value) => Clone::clone(value),
                    None => { 2 }
                },
            arity_nbits:
                match self.arity_nbits {
                    Some(ref value) => Clone::clone(value),
                    None =>
                    return Err(String::from("`arity_nbits` must be initialized")),
                },
            data:
                match self.data {
                    Some(ref value) => Clone::clone(value),
                    None =>
                    return Err(String::from("`data` must be initialized")),
                },
        }
            )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_shifts_like_cpp_version() {
        let mut bit: bool = true;
        let mut length = 0;
        let mut test: u8 = 1;
        for _ in 0..32 {
            dbg!(test);
            test |= (bit as u8) << super::BitString::bit_pos(length);
            length+=1;
            bit=!bit;
        }
        // From cpp_equivs/bit_string_shifts.cpp
        // (test=b:00000001,d:01) |= ((v=1) << (BITPOS(Length=0)=0) = rhs=b:00000001,d:01) = (test=b:00000001,d:01)
        // (test=b:00000001,d:01) |= ((v=0) << (BITPOS(Length=1)=1) = rhs=b:00000000,d:00) = (test=b:00000001,d:01)
        // (test=b:00000001,d:01) |= ((v=1) << (BITPOS(Length=2)=2) = rhs=b:00000100,d:04) = (test=b:00000101,d:05)
        // (test=b:00000101,d:05) |= ((v=0) << (BITPOS(Length=3)=3) = rhs=b:00000000,d:00) = (test=b:00000101,d:05)
        // (test=b:00000101,d:05) |= ((v=1) << (BITPOS(Length=4)=4) = rhs=b:00010000,d:16) = (test=b:00010101,d:21)
        // (test=b:00010101,d:21) |= ((v=0) << (BITPOS(Length=5)=5) = rhs=b:00000000,d:00) = (test=b:00010101,d:21)
        // (test=b:00010101,d:21) |= ((v=1) << (BITPOS(Length=6)=6) = rhs=b:01000000,d:64) = (test=b:01010101,d:85)
        // (test=b:01010101,d:85) |= ((v=0) << (BITPOS(Length=7)=7) = rhs=b:00000000,d:00) = (test=b:01010101,d:85)
        // (test=b:01010101,d:85) |= ((v=1) << (BITPOS(Length=8)=0) = rhs=b:00000001,d:01) = (test=b:01010101,d:85)
        // (test=b:01010101,d:85) |= ((v=0) << (BITPOS(Length=9)=1) = rhs=b:00000000,d:00) = (test=b:01010101,d:85)
        assert_eq!(test,85);
    }
}