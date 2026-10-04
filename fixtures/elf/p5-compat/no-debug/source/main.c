/*
 * FirmwareSight P5 compatibility fixture source. Written for this project.
 *
 * Plain FLASH and RAM placement, used where the case under test is the compiler or the presence of debug information rather than the layout.
 */

/* read-only: load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;


int main(void) {
    return (int)cfg_table[0] + g_scratch;
}
