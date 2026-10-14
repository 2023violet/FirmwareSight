/* Synthetic public test facts; no device/customer code. */
const unsigned char payload[4] = {1, 2, 3, 4};
volatile unsigned int initialized = 7;
volatile unsigned int zeroed;
unsigned int fixture_function(void) { return initialized + zeroed + payload[0]; }
