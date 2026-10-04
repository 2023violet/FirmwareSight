/*
 * FirmwareSight P5 compatibility fixture source. Written for this project.
 *
 * `.extsram_pool` is placed in a region above every internal-RAM address, so the region is knowable only from the linker's own memory map.
 */

/* read-only: load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

__attribute__((section(".extsram_pool"), used)) unsigned int ext_pool[32];
int main(void) {
    return (int)ext_pool[0] + g_scratch;
}
