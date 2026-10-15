/* Written for this project; no third-party code copied. */
static volatile int uart_tx;

void uart_send(int byte)
{
    uart_tx = byte;
}
