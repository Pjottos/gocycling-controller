#![no_std]
#![feature(asm)]

use crate::{
    cycling::CycleData,
    host::HostInterface,
};
use core::panic::PanicInfo;
use rpi_pico_sdk_sys::*;

mod binding;
mod ctypes;

mod host;
mod cycling;

const ONBOARD_LED_PIN: u32 = 25;
const MAGNET_PIN: u32 = 2;

const MODULES_STARTUP_MS: u32 = 350;


#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    sleep_ms(MODULES_STARTUP_MS);

    init_magnet_sensor();
    let mut host = HostInterface::new().unwrap();
    host.start(host::OperatingMode::Online);

    let mut last_cycle_time = time_us_64();

    loop {
        // asm!("wfi");
        // TODO: temporary until magnet sensor is figured out
        sleep_ms(1);
        let delta = time_us_64() - last_cycle_time;
        last_cycle_time = time_us_64();
        let data = CycleData { micros: delta as u32 };
        host.push(&data).unwrap();
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    unsafe {
        gpio_set_function(ONBOARD_LED_PIN, GPIO_OUT);
        gpio_put(ONBOARD_LED_PIN, true);
    }

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
