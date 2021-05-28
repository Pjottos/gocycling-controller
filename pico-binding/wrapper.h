#pragma once


void *init_uart0(unsigned int baud_rate, unsigned int tx_pin, unsigned int rx_pin);
void print_uart0(void* uart, const char *str);
