use crate::{binding::*, critical::CriticalSection, host, offline};
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Default)]
pub struct CycleData {
    pub millis: u32,
}

static mut LAST_CYCLE_TIME: u64 = 0;

pub fn handle_cycle(cs: &CriticalSection) {
    const MIN_CYCLE_DELTA: u64 = 50_000;

    let time = unsafe { time_us_64() };
    let delta = time - unsafe { LAST_CYCLE_TIME };
    if delta < MIN_CYCLE_DELTA {
        return;
    }

    unsafe {
        LAST_CYCLE_TIME = time;
    }
    let data = CycleData {
        millis: (delta / 1000) as u32,
    };

    if let Some(host) = unsafe { host::HOST_INTERFACE.as_mut() } {
        if host.has_connection(cs) {
            host.push_cycle(cs, data).ok();
        } else {
            offline::add_cycle(cs, &data).ok();
        }
    }
}

pub fn reset(_: &CriticalSection) {
    unsafe {
        LAST_CYCLE_TIME = time_us_64();
    }
}
