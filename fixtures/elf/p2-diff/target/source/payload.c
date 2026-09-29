/*
 * FirmwareSight P2 Compare fixture — the TARGET side only.
 *
 * Three deliberate differences, each visible to one acceptance item of US-002:
 *   `mqtt_task`        — a larger body, so its size grows and `tls_handshake` moves with it.
 *   `packet_router`    — a symbol this build has and the base does not: an Added symbol.
 *   `staging_area`     — a `.ota_staging` input section the target linker script places in RAM,
 *                        loaded from ROM: an Added section that charges BOTH memory budgets.
 * `g_frame_counter` grows `.bss` without touching the load image, which is the runtime-only case.
 * The base's `calib_apply` and `.calib` are absent here, which is the Removed case.
 */

__attribute__((section(".ota_staging"), used)) volatile unsigned char staging_area[64] = {
    0xA0u, 0xA1u, 0xA2u, 0xA3u, 0xA4u, 0xA5u, 0xA6u, 0xA7u, 0xA8u, 0xA9u, 0xAAu,
    0xABu, 0xACu, 0xADu, 0xAEu, 0xAFu, 0xB0u, 0xB1u, 0xB2u, 0xB3u, 0xB4u, 0xB5u,
    0xB6u, 0xB7u, 0xB8u, 0xB9u, 0xBAu, 0xBBu, 0xBCu, 0xBDu, 0xBEu, 0xBFu};

extern volatile unsigned char staging_area[64];
extern int g_scratch;
int g_frame_counter;

__attribute__((noinline)) int calib_apply(int seed) {
    /* No calibration blob in this build: the entry point stays, its data is gone. */
    return seed;
}

__attribute__((noinline, used)) int mqtt_task(int seed) {
    int acc = seed + 1;
    acc = (acc * 3) ^ 0x5A;
    acc += (acc >> 2) - g_scratch;
    acc = (acc * 5) ^ 0x3C;
    return acc + staging_area[3];
}

/* Byte-for-byte the same body as the base's. Only its address differs. */
__attribute__((noinline, used)) int tls_handshake(int seed) {
    return (seed ^ 0xAA) + g_scratch;
}

__attribute__((noinline, used)) int packet_router(int seed) {
    int hop = seed + g_frame_counter;
    hop = (hop * 7) ^ 0x1F;
    hop += (int)staging_area[hop & 63];
    return hop + g_scratch;
}

__attribute__((noinline)) int payload_extra(int seed) {
    return calib_apply(seed) + packet_router(seed);
}
