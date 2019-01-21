#[derive(Default, Builder, Clone)]
#[builder(setter(prefix = "with"))]
pub struct BitString {
    /// the number of bits in Data
    #[builder(default = "0")]
    length: u32,
    /// the arity that will be used for getLength/getNAry/appendNAry
    /// Arity is also seen as unsigned char which seems to be u8
    #[builder(setter(skip))]
    #[builder(default = "2")]
    arity: u8, // Do setter skip and default work together?
    /// the number of Bits per n-ary digit (where n is Arity)
    arity_nbits: u16, // setArity requires some more logic
    /// the actual data
    data: Vec<u8>,
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
impl BitStringBuilder{
    pub fn with_arity(&mut self, value: u8) -> &mut Self {
        let mut new = self;
        let tmp = value;
        let arity_nbits = 0u16;
        new.arity = Some(value);
        while(tmp > 1){
            if tmp % 2 != 0 {
                tmp /= 2;
                arity_nbits += 1;
            }
        }
        self.arity_nbits = Some(arity_nbits);
        new
    }
    /*

	while (tmp > 1) {
		myassert (tmp % 2 == 0) ; // only implemented for arity = 2^i
		tmp /= 2 ;
		ArityNBits++ ;
	}
    */
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