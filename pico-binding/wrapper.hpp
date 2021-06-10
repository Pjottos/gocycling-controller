#include "boards/pico.h"
#include "pico/stdio.h"
#include "pico/stdlib.h"
#include "hardware/sync.h"
#include "hardware/irq.h"
#include "hardware/uart.h"
#include "hardware/gpio.h"
#include "hardware/rtc.h"
#include "hardware/pwm.h"

extern "C" void *binding_uart0_init(uint baud_rate, uint tx_pin, uint rx_pin);
extern "C" void binding_uart_destroy(void* uart);
extern "C" void binding_uart_write_blocking(void* uart, const uint8_t *data, uint len);
extern "C" void binding_uart_read_blocking(void *uart, uint8_t *data, uint len);
extern "C" void binding_uart_set_irq_enables(void *uart, bool rx, bool tx);
extern "C" bool binding_uart_is_readable(void *uart);
extern "C" uint8_t binding_uart_getc(void *uart);

extern "C" void binding_irq_set_exclusive_handler(uint irq, void (*fn)());
extern "C" void binding_irq_set_enabled(uint irq, bool enabled);

extern "C" void binding_gpio_set_dir(uint gpio, bool out);
extern "C" void binding_gpio_put(uint gpio, bool value);
extern "C" bool binding_gpio_get(uint gpio);

extern "C" uint binding_pwm_gpio_to_slice_num(uint gpio);
extern "C" pwm_config binding_pwm_get_default_config();
extern "C" void binding_pwm_init(uint slice_num, pwm_config *config, bool running);
extern "C" void binding_pwm_set_gpio_level(uint gpio, uint16_t level);

extern "C" uint32_t binding_save_and_disable_interrupts();
extern "C" void binding_restore_interrupts(uint32_t status);
