/*
 * FirmwareSight P5 compatibility fixture source. Written for this project.
 *
 * One section, `.fast_text`, is executable and resident in RAM while its load address stays in FLASH, which is the shape that makes VMA and LMA disagree for code.
 */

/* read-only: load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

__attribute__((section(".fast_text"), used)) int ram_resident_fn(int x) {
    return x + g_threshold;
}
int main(void) {
    return ram_resident_fn((int)cfg_table[1]) + g_scratch;
}
