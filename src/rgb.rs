use crate::binding::*;

pub static mut BATTERY_LED: RgbLed = RgbLed {
    r_pin: 10,
    g_pin: 11,
    b_pin: 12,
};

pub static mut STATUS_LED: RgbLed = RgbLed {
    r_pin: 13,
    g_pin: 14,
    b_pin: 15,
};

pub struct RgbLed {
    r_pin: u32,
    g_pin: u32,
    b_pin: u32,
}

impl RgbLed {
    pub unsafe fn init(led: &RgbLed) {
        binding_gpio_set_dir(led.r_pin, true);
        binding_gpio_set_dir(led.g_pin, true);
        binding_gpio_set_dir(led.b_pin, true);

        gpio_set_function(led.r_pin, GPIO_FUNC_PWM);
        gpio_set_function(led.g_pin, GPIO_FUNC_PWM);
        gpio_set_function(led.b_pin, GPIO_FUNC_PWM);
    }

    /// All values are normalized, 
    pub fn put_hsv(&self, h: u8, s: u8, v: u8) {
        let (r, g, b) = hsv_to_rbg(h, s, v);
        self.put_rgb(r, g, b);
    }

    pub fn put_rgb(&self, r: u16, g: u16, b: u16) {
        todo!()
    }
}

fn hsv_to_rbg(h: u8, s: u8, v: u8) -> (u16, u16, u16) {
    let mut numerator = v as u16 * (!s) as u16;
    numerator += numerator / 256;
    numerator += 1;

    todo!();
}
