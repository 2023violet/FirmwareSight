/*
 * FirmwareSight P2 Compare fixture — the BASE side only.
 *
 * `calib_apply` and the `.calib` blob exist in this build and not in the target, so Compare has a
 * genuine Removed symbol and a genuine Removed section to report. `mqtt_task` is the small version:
 * the target's larger body is what makes it a Changed-size symbol.
 */

__attribute__((section(".calib"), used)) const unsigned char calib[32] = {
    0x10u, 0x11u, 0x12u, 0x13u, 0x14u, 0x15u, 0x16u, 0x17u, 0x18u, 0x19u, 0x1au,
    0x1bu, 0x1cu, 0x1du, 0x1eu, 0x1fu, 0x20u, 0x21u, 0x22u, 0x23u, 0x24u, 0x25u,
    0x26u, 0x27u, 0x28u, 0x29u, 0x2au, 0x2bu, 0x2cu, 0x2du, 0x2eu, 0x2fu};

extern const unsigned char calib[32];
extern int g_scratch;

__attribute__((noinline)) int calib_apply(int seed) {
    return (int)calib[seed & 31] + seed;
}

__attribute__((noinline, used)) int mqtt_task(int seed) {
    return seed + 1;
}

/* Same body as the target's. Its address still moves, because mqtt_task ahead of it grows. */
__attribute__((noinline, used)) int tls_handshake(int seed) {
    return (seed ^ 0xAA) + g_scratch;
}

__attribute__((noinline)) int payload_extra(int seed) {
    return calib_apply(seed);
}
