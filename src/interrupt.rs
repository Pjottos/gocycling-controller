use crate::binding::*;

pub unsafe fn init() {
    binding_irq_set_exclusive_handler(10, Some(on_gpio));
    // pin number has no effect
    gpio_set_irq_enabled(
        0,
        GPIO_IRQ_EDGE_FALL,
        true,
    );
}


extern "C" fn on_gpio() {

}
