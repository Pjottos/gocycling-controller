#![no_std]
#![feature(asm)]

use crate::cycling::CycleData;
use core::panic::PanicInfo;
use rpi_pico_sdk_sys::*;

mod binding;
mod ctypes;

mod host;
mod cycling;

const MAGNET_PIN: u32 = 2;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let conn = host::Connection::new().unwrap();

    unsafe { init_magnet_sensor(); }
    let mut last_cycle_time = unsafe { time_us_64() };

    loop {
        unsafe {
            asm!("wfi");
            let delta = time_us_64() - last_cycle_time;
            last_cycle_time = time_us_64();
            let data = CycleData { micros: delta as u32 };
            conn.push(&data).unwrap();
        }
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe fn init_magnet_sensor() {
    gpio_set_dir(MAGNET_PIN, GPIO_IN);
    gpio_pull_up(MAGNET_PIN);
    gpio_set_irq_enabled(
        0,
        gpio_irq_level_GPIO_IRQ_EDGE_FALL,
        true,
    );
}
