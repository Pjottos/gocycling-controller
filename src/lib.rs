#![no_std]
#![feature(asm)]

use crate::{
    binding::*,
    cycling::CycleData,
    host::HostInterface,
};
use core::panic::PanicInfo;
use rgb::RgbLed;

mod binding;
mod ctypes;

mod host;
mod cycling;
mod rgb;
mod interrupt;


const MODULES_STARTUP_MS: u32 = 350;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    sleep_ms(MODULES_STARTUP_MS);

    RgbLed::init(&rgb::STATUS_LED);
    RgbLed::init(&rgb::BATTERY_LED);
    HostInterface::create();
    interrupt::init();

    let host = host::HOST_INTERFACE.as_mut().unwrap();
    let mode = wait_for_mode_select();
    host.start(mode);

    let mut last_cycle_time = time_us_64();

    loop {
        // asm!("wfi");
        // TODO: temporary until magnet sensor is figured out
        sleep_ms(1000);
        let delta = time_us_64() - last_cycle_time;
        last_cycle_time = time_us_64();
        let data = CycleData { micros: delta as u32 };
        host.push(&data).unwrap();
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    const PIN_ONBOARD_LED: u32 = 25;

    unsafe {
        gpio_set_function(PIN_ONBOARD_LED, GPIO_OUT);
        binding_gpio_put(PIN_ONBOARD_LED, true);

        loop {
            sleep_ms(1);
        }
    }
}

unsafe fn wait_for_mode_select() -> host::OperatingMode {
    // TODO
    return host::OperatingMode::Online;

    const TICK_US: u64 = 500;
    const H_PER_TICK: u8 = 1;

    let mut h = 0;
    loop {
        rgb::STATUS_LED.put_hsv(h, u8::MAX, u8::MAX);

        h = h.overflowing_add(H_PER_TICK).0;
        sleep_us(TICK_US);
    }
}
