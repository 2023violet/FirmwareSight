/*
 * FirmwareSight P2 Compare fixture — the half that is byte-identical in both builds.
 *
 * Nothing in this file changes between the base and the target build, so every fact it produces is
 * a control: `.rodata` and `.data` keep their sizes, and `boot_check` and `main` keep both their
 * size and their address because this object is linked first. Compare must report those as
 * unchanged, and must not report a row position as a change.
 *
 * Deliberately not product demo data: the numbers are here to be checkable.
 */

/* read-only: contributes to the load image only, identical in both builds */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM, identical in both builds */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

/*
 * A settings blob at a fixed flash address, identical in both builds. The linker script pins its
 * address, so it keeps its size, its VMA and its LMA while the code around it moves. Compare needs
 * a genuinely unchanged section to prove it can say "no change" — every other section in this pair
 * shifts somewhere.
 */
__attribute__((section(".settings"), used)) const unsigned char settings_page[64] = {
    'F',  'W',  'S',  0u,   1u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u};

/*
 * Kept alive from main() so -Os cannot drop it. Its address is fixed by link order, which is what
 * makes it the unchanged-symbol control.
 */
__attribute__((noinline, used)) int boot_check(void) {
    return g_threshold ^ (int)cfg_table[0];
}

int mqtt_task(int seed);
int tls_handshake(int seed);
int payload_extra(int seed);
int calib_apply(int seed);

__attribute__((noinline)) int main_entry(int seed);

__attribute__((noinline)) int main_entry(int seed) {
    int sum = boot_check() + mqtt_task(seed);
    sum += tls_handshake(seed);
    sum += payload_extra(seed);
    sum += calib_apply(seed);
    return sum + g_scratch;
}

int main(void) {
    return main_entry(g_threshold);
}
