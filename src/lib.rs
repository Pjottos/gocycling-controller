#![no_std]

use binding::*;
use core::panic::PanicInfo;
use rpi_pico_sdk_sys::*;

mod binding;
mod ctypes;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    const PIN_LED: u32 = 25;

    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    let uart = init_uart0(BAUD_RATE, TX_PIN, RX_PIN);

    loop {
        print_uart(uart, b"what\0".as_ptr());
        sleep_ms(1000);
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}
