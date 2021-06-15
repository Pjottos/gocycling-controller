#![no_std]
#![feature(asm)]

use crate::{binding::*, host::HostInterface, state::ProgramState};
use core::panic::PanicInfo;

mod binding;
mod ctypes;

mod critical;
mod cycling;
mod host;
mod interrupt;
mod rgb;
mod state;
mod offline;

const MODULES_STARTUP_MS: u32 = 250;

const PIN_STATUS_LED_R: u32 = 6;
const PIN_STATUS_LED_G: u32 = 7;
const PIN_STATUS_LED_B: u32 = 8;

const PIN_BATTERY_LED_R: u32 = 2;
const PIN_BATTERY_LED_G: u32 = 3;
const PIN_BATTERY_LED_B: u32 = 4;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    interrupt::init();

    // sleep_ms(MODULES_STARTUP_MS);

    HostInterface::create();
    rtc_init();

    let status_led = rgb::RgbLed::new(
        PIN_STATUS_LED_R,
        PIN_STATUS_LED_G,
        PIN_STATUS_LED_B,
    );
    let mut old_status_hue: u8 = 0;
    let battery_led: rgb::RgbLed = rgb::RgbLed::new(
        PIN_BATTERY_LED_R,
        PIN_BATTERY_LED_G,
        PIN_BATTERY_LED_B,
    );

	let mut rainbow_hue = 0;
    loop {
        let state = critical::run(|cs| state::retrieve(cs));
        match state {
            ProgramState::WaitForModeSelect => {
                const TICK_MS: u32 = 3;
                const HUE_PER_TICK: u8 = 1;
                status_led.put_rainbow_hue(rainbow_hue);
                old_status_hue = rainbow_hue;

                rainbow_hue  = rainbow_hue.overflowing_add(HUE_PER_TICK).0;
                sleep_ms(TICK_MS);

                critical::run(|cs| {
                    // don't overwrite a running state
                    if let ProgramState::WaitForModeSelect = state::retrieve(cs) {
                        state::store(cs, ProgramState::WaitForModeSelect);
                    }
                });
            },
            ProgramState::Running { status_hue } => {
                if status_hue != old_status_hue {
                    status_led.put_rainbow_hue(status_hue);
                    old_status_hue = status_hue;
                }

                asm!("wfe");
            }
        }
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    const PIN_ONBOARD_LED: u32 = 25;

    unsafe {
        gpio_set_function(PIN_ONBOARD_LED, GPIO_OUT);
        binding_gpio_put(PIN_ONBOARD_LED, true);

        loop {
            asm!("wfe");
        }
    }
}

