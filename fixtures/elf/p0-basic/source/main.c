/*
 * FirmwareSight P0 technical fixture.
 *
 * Each object below is placed in a different memory class on purpose so that section
 * classification, symbol tables and the two memory budgets can be asserted against a real
 * linker's output rather than against a description of one.
 *
 * Deliberately not product demo data: the numbers are here to be checkable.
 */

/* read-only: contributes to the load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

/* custom region, explicitly placed by the linker script */
__attribute__((section(".ota_staging"), used)) unsigned char staging_area[64] = {9u, 8u, 7u};

int mqtt_task(void) {
    return g_threshold + (int)cfg_table[0] + g_scratch + (int)staging_area[3];
}

int tls_handshake(void) {
    return mqtt_task() ^ 0xAA;
}

int sensor_fifo(void) {
    return tls_handshake() + 1;
}

int main(void) {
    return sensor_fifo();
}
