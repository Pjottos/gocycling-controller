#![no_std]
#![no_main]

#![feature(asm)]

use cortex_m_rt as rt;
use rt::entry;
use rp2040_hal::{
    prelude::*,
    pac::Peripherals,
    uart,
};
use embedded_time::rate::Hertz;
use embedded_hal::{
    prelude::*,
    digital::v2::*,
};

use panic_halt as _;


#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

#[entry]
fn main() -> ! {
    let mut pac = Peripherals::take().unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = pac
        .IO_BANK0
        .split(pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);
    let mut led_pin = pins.gpio25.into_output();

    let mut uart = uart::UartPeripheral::enable(
        pac.UART0,
        uart::common_configs::_115200_8_N_1,
        Hertz(1000),
    )
    .unwrap();

    let mut var = true;
    
    loop {
        uart.write(b'a').unwrap();

        if var {
            led_pin.set_high().unwrap();
        }
        else {
            led_pin.set_low().unwrap();
        }

        for _ in 0..100000 {
            unsafe { asm!("nop") }
        }

        var = !var;
    }
}
