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
    rgb::STATUS_LED.init();
    rgb::BATTERY_LED.init();

    sleep_ms(MODULES_STARTUP_MS);

    HostInterface::create();
    interrupt::init();
    rtc_init();

    let mode = wait_for_mode_select();

    let host = host::HOST_INTERFACE.as_mut().unwrap();
    host.start(mode);

    loop {
        sleep_ms(1);
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
    return host::OperatingMode::Online {
        started: true,
        connected: true,
    };

    const TICK_MS: u32 = 3;
    const H_PER_TICK: u8 = 1;

    let mut hue = 0;
    loop {
        rgb::STATUS_LED.put_rainbow_hue(hue);

        hue = hue.overflowing_add(H_PER_TICK).0;
        sleep_ms(TICK_MS);
    }
}
