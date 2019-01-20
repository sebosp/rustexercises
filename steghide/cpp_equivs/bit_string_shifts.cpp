#include <stdio.h>
#include <iostream>
#include <bitset>
typedef unsigned long	UWORD32 ;
static const UWORD32 Magic = 0x73688DUL;
UWORD32 Length ;

#define BITPOS(n) (n % 8)
#define BYTEPOS(n) (n / 8)

typedef unsigned char	BYTE ;

int main(int argc, char *argv[])
{
	BYTE test = 1;
	bool v = true;
	Length = 0;
	for(int i = 0; i < 32; i++)
	{
		std::cout << "(test=b:" <<  std::bitset<8>(test);
		printf(",d:%02u) |= ", test);
		std::cout << "((v=" << v << ") << (BITPOS(Length=" << Length << ")=";
		std::cout << BITPOS(Length) << ") = rhs=b:" << std::bitset<8>(v << BITPOS(Length));
		printf(",d:%02u) = ", (v << BITPOS(Length)));
		test |= (v << BITPOS(Length));
		std::cout << "(test=b:" <<  std::bitset<8>(test);
		printf(",d:%02u)\n", test);
		Length++;
		v=!v;
	}
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
	return 0;
}
