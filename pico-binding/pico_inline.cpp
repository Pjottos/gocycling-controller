#include "wrapper.hpp"

void *binding_uart0_init(uint baud_rate, uint tx_pin, uint rx_pin) {
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

void binding_uart_write_blocking(void *uart, const uint8_t *data, uint len) {
    uart_write_blocking((uart_inst_t *)uart, data, len);
}

void binding_uart_read_blocking(void *uart, uint8_t *data, uint len) {
    uart_read_blocking((uart_inst_t *)uart, data, len);
}

void binding_uart_set_irq_enables(void *uart, bool rx, bool tx) {
    uart_set_irq_enables((uart_inst_t *)uart, rx, tx);
}

bool binding_uart_is_readable(void *uart) {
    return uart_is_readable((uart_inst_t *)uart);
}

uint8_t binding_uart_getc(void *uart) {
    return uart_getc((uart_inst_t *)uart);
}


void binding_irq_set_exclusive_handler(uint irq, void (*fn)()) {
    irq_set_exclusive_handler(irq, fn);
}

void binding_irq_set_enabled(uint irq, bool enabled) {
    irq_set_enabled(irq, enabled);
}


void binding_gpio_set_dir(uint gpio, bool out) {
    gpio_set_dir(gpio, out);
}

void binding_gpio_put(uint gpio, bool value) {
    gpio_put(gpio, value);
}

bool binding_gpio_get(uint gpio) {
    return gpio_get(gpio);
}


uint binding_pwm_gpio_to_slice_num(uint gpio) {
    return pwm_gpio_to_slice_num(gpio);
}

pwm_config binding_pwm_get_default_config() {
    return pwm_get_default_config();
}

void binding_pwm_init(uint slice_num, pwm_config *config, bool running) {
    pwm_init(slice_num, config, running);
}

void binding_pwm_set_gpio_level(uint gpio, uint16_t level) {
    pwm_set_gpio_level(gpio, level);
}


uint32_t binding_save_and_disable_interrupts() {
    return save_and_disable_interrupts();
}

void binding_restore_interrupts(uint32_t status) {
    restore_interrupts(status);
}
