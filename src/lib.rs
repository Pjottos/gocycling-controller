#![no_std]

use rpi_pico_sdk_sys::*;

use core::panic::PanicInfo;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    const PIN_LED: u32 = 25;

    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    // uart_init(uart0, BAUD_RATE);

    gpio_set_function(TX_PIN, GPIO_FUNC_UART);
    gpio_set_function(RX_PIN, GPIO_FUNC_UART);

    gpio_init(PIN_LED);
    gpio_set_dir(PIN_LED, GPIO_OUT);

    loop {
        gpio_put(PIN_LED, true);
        sleep_ms(500);
        gpio_put(PIN_LED, false);
        sleep_ms(500);
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}
