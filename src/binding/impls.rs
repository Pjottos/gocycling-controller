use super::*;

impl datetime_t {
    pub fn from_bits(bits: u64) -> Self {
        Self {
            year:   ((bits >> 0 ) & 0x0FFF) as i16,
            month:  ((bits >> 12) & 0x000F) as i8,
            day:    ((bits >> 16) & 0x001F) as i8,
            hour:   ((bits >> 21) & 0x001F) as i8,
            min:    ((bits >> 26) & 0x003F) as i8,
            sec:    ((bits >> 32) & 0x003F) as i8,
            dotw: 0,
        }
    }
}
