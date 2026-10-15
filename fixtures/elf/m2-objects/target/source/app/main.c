/* Written for this project; no third-party code copied. */
extern int imu_read(int axis);
extern void uart_send(int byte);
extern void spi_flush(void);
int g_samples;                 /* uninitialized: placed by -fcommon as a COMMON symbol */
static volatile int g_threshold = 42;   /* initialized: .data, charged to Flash and RAM */

void Reset_Handler(void)
{
    for (int i = 0; i < 3; i++) {
        int v = imu_read(i);
        if (v > g_threshold) {
            uart_send(v);
        }
        g_samples += v;
    }
    spi_flush();
    for (;;) {
    }
}

int unused_helper(int x)       /* never referenced: discarded by --gc-sections */
{
    return x * 3;
}
