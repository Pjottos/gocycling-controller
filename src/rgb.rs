use crate::binding::*;

pub const MAX_BRIGHTNESS: u16 = 0x0CFF;

pub struct RgbLed {
    r_pin: u32,
    g_pin: u32,
    b_pin: u32,
}

impl RgbLed {
    pub unsafe fn new(r_pin: u32, g_pin: u32, b_pin: u32) -> Self {
        Self::init_pin(r_pin);
        Self::init_pin(g_pin);
        Self::init_pin(b_pin);

        let led = RgbLed {
            r_pin,
            g_pin,
            b_pin,
        };

        // turn off until the led is needed
        led.put_rgb(0, 0, 0);

        led
    }

    unsafe fn init_pin(pin: u32) {
        gpio_set_function(pin, GPIO_FUNC_PWM);
        let slice_num = binding_pwm_gpio_to_slice_num(pin);
        let mut config = binding_pwm_get_default_config();
        binding_pwm_init(slice_num, &mut config, true);
    }

    pub fn put_rainbow_hue(&self, hue: u8) {
        let (r, g, b) = hue_to_rgb_rainbow(hue);
        let (r, g, b) = scale_rgb(r, g, b);
        self.put_rgb(r, g, b);
    }

    // Values are clamped with a max of [MAX_BRIGHTNESS]
    pub fn put_rgb(&self, r: u16, g: u16, b: u16) {
        // limit the max brightness of the led
        let r = r.clamp(0, MAX_BRIGHTNESS);
        let g = g.clamp(0, MAX_BRIGHTNESS);
        let b = b.clamp(0, MAX_BRIGHTNESS);

        unsafe {
            // we're using a common anode led, so the level needs to be
            // inverted for current sinking
            binding_pwm_set_gpio_level(self.r_pin, u16::MAX - r);
            binding_pwm_set_gpio_level(self.g_pin, u16::MAX - g);
            binding_pwm_set_gpio_level(self.b_pin, u16::MAX - b);
        }
    }
}

#[rustfmt::skip]
fn hue_to_rgb_rainbow(hue: u8) -> (u8, u8, u8) {
    // Divide the hue range into 8 sections (3 bits)
    let section = (hue & 0xE0) >> 5;
    // The rest of the hue value will be used for the offset into the section
    let section_offset = hue & 0x1F;

    // Various constants useful for calculating the RGB channel values
    const CONST_85: u8 = (256 / 3) as u8;
    const CONST_170: u8 = CONST_85 * 2;

    // Scale the section offset to use it for the channel values
    let offset = scale_u8(section_offset << 3, CONST_85);
    let offset_2 = scale_u8(section_offset << 3, CONST_170);

    match section {
        // red -> orange
        0 => (
            u8::MAX - offset,
            offset,
            0,
        ),
        // orange -> yellow
        1 => (
            CONST_170,
            CONST_85 + offset,
            0,
        ),
        // yellow -> green
        2 => (
            CONST_170 - offset_2,
            CONST_170 + offset,
            0,
        ),
        // green -> cyan
        3 => (
            0,
            u8::MAX - offset,
            offset,
        ),
        // cyan -> blue
        4 => (
            0,
            CONST_170 - offset_2,
            CONST_85 + offset_2,
        ),
        // blue -> purple
        5 => (
            offset,
            0,
            u8::MAX - offset,
        ),
        // purple -> pink
        6 => (
            CONST_85 + offset,
            0,
            CONST_170 - offset,
        ),
        // pink -> red
        7 => (
            CONST_170 + offset,
            0,
            CONST_85 - offset,
        ),
        _ => unreachable!(),
    }
}

fn scale_rgb(r: u8, g: u8, b: u8) -> (u16, u16, u16) {
    (
        u16::from(r) * u16::max(1, MAX_BRIGHTNESS / u16::from(u8::MAX)),
        u16::from(g) * u16::max(1, MAX_BRIGHTNESS / u16::from(u8::MAX)),
        u16::from(b) * u16::max(1, MAX_BRIGHTNESS / u16::from(u8::MAX)),
    )
}

fn scale_u8(value: u8, scale: u8) -> u8 {
    let big = u16::from(value) * (1 + u16::from(scale));
    (big / 256) as u8
}
