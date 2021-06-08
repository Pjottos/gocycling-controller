#include "boards/pico.h"
#include "pico/stdio.h"
#include "pico/stdlib.h"
#include "hardware/irq.h"
#include "hardware/uart.h"
#include "hardware/gpio.h"

extern "C" void *binding_uart0_init(uint baud_rate, uint tx_pin, uint rx_pin);
extern "C" void binding_uart_destroy(void* uart);
extern "C" void binding_uart_write_blocking(void* uart, const uint8_t *data, uint len);
extern "C" void binding_uart_read_blocking(void *uart, uint8_t *data, uint len);
extern "C" void binding_uart_set_irq_enables(void *uart, bool rx, bool tx);

extern "C" void binding_irq_set_exclusive_handler(uint irq, void (*fn)());
extern "C" void binding_irq_set_enabled(uint irq, bool enabled);

extern "C" void binding_gpio_set_dir(uint gpio, bool out);
extern "C" void binding_gpio_put(uint gpio, bool value);
extern "C" bool binding_gpio_get(uint gpio);
