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

const PIN_ONBOARD_LED: u32 = 25;

const PIN_MAGNET: u32 = 2;

const PIN_LED_BATTERY_R: u32 = 0;
const PIN_LED_BATTERY_G: u32 = 0;
const PIN_LED_BATTERY_B: u32 = 0;

const PIN_LED_STATUS_R: u32 = 0;
const PIN_LED_STATUS_G: u32 = 0;
const PIN_LED_STATUS_B: u32 = 0;

const PIN_BATTERY_LEVEL_IN: u32 = 0;

const MODULES_STARTUP_MS: u32 = 350;


#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    sleep_ms(MODULES_STARTUP_MS);

    init_pins();
    let mut host = HostInterface::new().unwrap();

    let battery_led = RgbLed::new(
        PIN_LED_BATTERY_R,
        PIN_LED_BATTERY_G,
        PIN_LED_BATTERY_B,
    );
    let status_led = RgbLed::new(
        PIN_LED_STATUS_R,
        PIN_LED_STATUS_G,
        PIN_LED_STATUS_B,
    );
    let mode = host::OperatingMode::Online; // wait_for_mode_select(&status_led);

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
    unsafe {
        gpio_set_function(PIN_ONBOARD_LED, GPIO_OUT);
        binding_gpio_put(PIN_ONBOARD_LED, true);

        loop {
            sleep_ms(1);
        }
    }
}

unsafe fn init_pins() {
    binding_gpio_set_dir(PIN_MAGNET, false);
    gpio_set_pulls(PIN_MAGNET, true, false);

    // TODO: set up adc for battery level
}

unsafe fn wait_for_mode_select(status_led: &RgbLed) -> host::OperatingMode {
    const TICK_US: u64 = 500;
    const H_PER_TICK: u8 = 1;

    let mut h = 0;
    loop {
        status_led.put_hsv(h, u8::MAX, u8::MAX);

        h = h.overflowing_add(H_PER_TICK).0;
        sleep_us(TICK_US);
    }
}
