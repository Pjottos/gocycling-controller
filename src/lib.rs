#![no_std]
#![feature(asm)]

use crate::{binding::*, host::HostInterface};
use core::panic::PanicInfo;

mod binding;
mod ctypes;

mod critical;
mod cycling;
mod host;
mod interrupt;
mod rgb;

const MODULES_STARTUP_MS: u32 = 500;

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    rgb::STATUS_LED.init();
    rgb::BATTERY_LED.init();

    sleep_ms(MODULES_STARTUP_MS);

    HostInterface::create();
    interrupt::init();
    rtc_init();

    let mut state = ProgramState::WaitForModeSelect { hue: 0 };

    loop {
        state = state.execute();
    }
}

enum ProgramState {
    WaitForModeSelect { hue: u8 },
    Running,
}

impl ProgramState {
    unsafe fn execute(self) -> Self {
        match self {
            ProgramState::WaitForModeSelect { mut hue } => {
                const TICK_MS: u32 = 3;
                const HUE_PER_TICK: u8 = 1;

                rgb::STATUS_LED.put_rainbow_hue(hue);

                hue = hue.overflowing_add(HUE_PER_TICK).0;
                sleep_ms(TICK_MS);

                match host::HOST_INTERFACE.as_ref().unwrap().online() {
                    Some(true) => {
                        rgb::STATUS_LED.put_rgb(0, 0, u16::MAX);
                        ProgramState::Running
                    },
                    Some(false) => {
                        rgb::STATUS_LED.put_rgb(u16::MAX, 0, u16::MAX);
                        ProgramState::Running
                    }
                    None => {
                        ProgramState::WaitForModeSelect { hue }
                    }
                }
            },
            ProgramState::Running => ProgramState::Running,
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
            sleep_ms(1);
        }
    }
}
