/* Written for this project; no third-party code copied. */
static volatile int spi_buffer[8];

void spi_flush(void)
{
    for (int i = 0; i < 8; i++) {
        spi_buffer[i] = 0;
    }
}
