#pragma once


extern "C" void *init_uart0(unsigned int baud_rate, unsigned int tx_pin, unsigned int rx_pin);
extern "C" void print_uart(void* uart, const char *str);
