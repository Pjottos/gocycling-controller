#pragma once

#include <stdint.h>

extern "C" void *binding_uart0_init(uint32_t baud_rate, uint32_t tx_pin, uint32_t rx_pin);
extern "C" void binding_uart_destroy(void* uart);
extern "C" void binding_uart_write_blocking(void* uart, const uint8_t *data, uint32_t len);
extern "C" void binding_uart_read_blocking(void *uart, uint8_t *data, uint32_t len);