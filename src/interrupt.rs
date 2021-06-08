use crate::{
    binding::*,
    cycling,
    host::{HOST_INTERFACE, HostInterface},
};

const PIN_MAGNET_SENSOR: u32 = 2;
const PIN_BATTERY_LEVEL_IN: u32 = 26;

pub unsafe fn init() {
    binding_gpio_set_dir(PIN_MAGNET_SENSOR, false);
    gpio_set_pulls(PIN_MAGNET_SENSOR, true, false);

    // pin number has no effect
    gpio_set_irq_enabled_with_callback(
        0,
        GPIO_IRQ_EDGE_FALL,
        true,
        Some(on_gpio),
    );

    // TODO: battery level adc
}


unsafe extern "C" fn on_gpio(pin: u32, events: u32) {
    let edge = (events & 0xC) == GPIO_IRQ_EDGE_RISE;
    let _level = (events & 0x3) == GPIO_IRQ_LEVEL_HIGH;

    match pin {
        PIN_MAGNET_SENSOR => cycling::handle_cycle(),
        HostInterface::PIN_CONNECTION_STATE => if let Some(interface) = HOST_INTERFACE.as_mut() {
            interface.connection_changed(!edge);
        },
        _ => (),
    }
}
