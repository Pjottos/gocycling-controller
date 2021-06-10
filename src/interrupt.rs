use crate::{
    binding::*,
    cycling,
    host::HOST_INTERFACE,
};

const PIN_MAGNET_SENSOR: u32 = 5;
const PIN_BATTERY_LEVEL_IN: u32 = 26;
pub const PIN_CONNECTION_STATE: u32 = 21;

pub unsafe fn init() {
    binding_gpio_set_dir(PIN_MAGNET_SENSOR, false);
    gpio_set_pulls(PIN_MAGNET_SENSOR, true, false);

    binding_gpio_set_dir(PIN_CONNECTION_STATE, false);

    gpio_set_irq_enabled(
        PIN_CONNECTION_STATE,
        GPIO_IRQ_EDGE_FALL | GPIO_IRQ_EDGE_RISE,
        true,
    );
    gpio_set_irq_enabled(PIN_MAGNET_SENSOR, GPIO_IRQ_EDGE_FALL, true);

    // pin number and events params have no effect
    gpio_set_irq_enabled_with_callback(0, 0, true, Some(on_gpio));

    // TODO: battery level adc
}

unsafe extern "C" fn on_gpio(pin: u32, events: u32) {
    let edge = (events & 0xC) == GPIO_IRQ_EDGE_RISE;
    let _level = (events & 0x3) == GPIO_IRQ_LEVEL_HIGH;

    match pin {
        PIN_MAGNET_SENSOR => cycling::handle_cycle(),
        PIN_CONNECTION_STATE => {
            if let Some(interface) = HOST_INTERFACE.as_mut() {
                interface.connection_changed(!edge);
            }
        }
        _ => (),
    }
}
