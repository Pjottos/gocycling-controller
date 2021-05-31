#include "wrapper.hpp"

#include "boards/pico.h"
#include "pico/stdio.h"
#include "pico/stdlib.h"
#include "hardware/irq.h"
#include "hardware/uart.h"

void *init_uart0(unsigned int baud_rate, unsigned int tx_pin, unsigned int rx_pin) {
    uart_init(uart0, baud_rate);

    gpio_set_function(tx_pin, GPIO_FUNC_UART);
    gpio_set_function(rx_pin, GPIO_FUNC_UART);

    uart_set_hw_flow(uart0, false, false);
    uart_set_format(uart0, 8, 1, UART_PARITY_NONE);

    return uart0;
}


void print_uart(void *uart, const char *str) {
    uart_puts((uart_inst_t *)uart, str);
}

