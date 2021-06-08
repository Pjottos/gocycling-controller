use crate::binding::*;

use rpi_pico_sdk_sys::*;


pub struct RgbLed {
    r_pin: u32,
    g_pin: u32,
    b_pin: u32,
}

impl RgbLed {
    pub unsafe fn new(r_pin: u32, g_pin: u32, b_pin: u32) -> Self {
        gpio_set_dir(r_pin, GPIO_OUT);
        gpio_set_function(r_pin, gpio_function_GPIO_FUNC_PWM);
        gpio_set_dir(g_pin, GPIO_OUT);
        gpio_set_function(g_pin, gpio_function_GPIO_FUNC_PWM);
        gpio_set_dir(b_pin, GPIO_OUT);
        gpio_set_function(b_pin, gpio_function_GPIO_FUNC_PWM);

        Self {
            r_pin,
            g_pin,
            b_pin,
        }
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
