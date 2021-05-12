#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
extern fn main() {
    loop {

    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}
