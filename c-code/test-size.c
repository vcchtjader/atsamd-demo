#include "hpl/pukcc/CryptoLib_Headers_pb.h"

// Trick to get sizes of structs in compiler error messages
// Command: $ arm-none-eabi-gcc -c test-size.c -o t
char checker(int);
char checkSizeOfInt[sizeof(PUKCL_STATUS)]={checker(&checkSizeOfInt)};
char checkSizeOfInt[sizeof(PUKCL_HEADER)]={checker(&checkSizeOfInt)};
char checkSizeOfInt[sizeof(PUKCL_PARAM)]={checker(&checkSizeOfInt)};
