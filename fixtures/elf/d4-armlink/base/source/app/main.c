/* Written for this project; no third-party code copied. */
extern int imu_read(int axis);
extern void uart_send(int byte);

int g_samples;                 /* uninitialized: a common symbol with -fcommon */
static volatile int g_threshold = 42;   /* initialized: RW data, charged to Flash and RAM */

void Reset_Handler(void)
{
    for (int i = 0; i < 3; i++) {
        int v = imu_read(i);
        if (v > g_threshold) {
            uart_send(v);
        }
        g_samples += v;
    }

    for (;;) {
    }
}

int unused_helper(int x)       /* never referenced: removed by armlink --remove */
{
    return x * 3;
}
