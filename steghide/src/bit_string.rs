pub struct BitString {
	/// the number of bits in Data
	length: u32,
	/// the arity that will be used for getLength/getNAry/appendNAry
	arity: u8,
	/// the number of Bits per n-ary digit (where n is Arity)
	arity_nbits: u16,
	/// the actual data
	data: Vec<u8>,
}