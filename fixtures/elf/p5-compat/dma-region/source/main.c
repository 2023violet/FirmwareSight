/*
 * FirmwareSight P5 compatibility fixture source. Written for this project.
 *
 * `.dma_buffer` is placed in a region named for its use rather than for any name the product recognises. Nothing here claims anything about caches, because a linker script cannot.
 */

/* read-only: load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

__attribute__((section(".dma_buffer"), used)) unsigned int dma_ring[16];
int main(void) {
    return (int)dma_ring[0] + g_scratch;
}
