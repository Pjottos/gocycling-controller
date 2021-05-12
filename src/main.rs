#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
extern fn _start() {
    loop {

    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}
