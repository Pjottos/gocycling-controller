#include "wrapper.hpp"

#include "boards/pico.h"
#include "pico/stdio.h"
#include "pico/stdlib.h"
#include "hardware/irq.h"
#include "hardware/uart.h"

void *binding_uart0_init(uint32_t baud_rate, uint32_t tx_pin, uint32_t rx_pin) {
    uart_init(uart0, baud_rate);

    gpio_set_function(tx_pin, GPIO_FUNC_UART);
    gpio_set_function(rx_pin, GPIO_FUNC_UART);

    uart_set_hw_flow(uart0, false, false);
    uart_set_format(uart0, 8, 1, UART_PARITY_NONE);

    return uart0;
}

void binding_uart_destroy(void *uart) {
    uart_deinit((uart_inst_t *)uart);
}

void binding_uart_write_blocking(void *uart, const uint8_t *data, uint32_t len) {
    uart_write_blocking((uart_inst_t *)uart, data, len);
}

void binding_uart_read_blocking(void *uart, uint8_t *data, uint32_t len) {
    uart_read_blocking((uart_inst_t *)uart, data, len);
}
