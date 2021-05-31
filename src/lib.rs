#![no_std]

use crate::cycling::CycleData;
use core::panic::PanicInfo;
use rpi_pico_sdk_sys::*;

mod binding;
mod ctypes;

mod host;
mod cycling;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    const PIN_LED: u32 = 25;

    let conn = host::Connection::new().unwrap();
    let data = CycleData { time: 1.0 };

    loop {
        conn.push(&data).unwrap();
        sleep_ms(1000);
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}
