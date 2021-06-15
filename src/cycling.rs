use crate::{binding::*, critical::CriticalSection, host};
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Default)]
pub struct CycleData {
    pub micros: u32,
}

static mut LAST_CYCLE_TIME: u64 = 0;

pub unsafe fn handle_cycle(cs: &CriticalSection) {
    const MIN_CYCLE_DELTA: u64 = 50_000;

    let time = time_us_64();
    let delta = time - LAST_CYCLE_TIME;
    if delta < MIN_CYCLE_DELTA {
        return;
    }

    LAST_CYCLE_TIME = time;
    let data = CycleData {
        micros: delta as u32,
    };

    if let Some(host) = host::HOST_INTERFACE.as_mut() {
        host.push_cycle(cs, data).ok();
    }
}

pub unsafe fn reset() {
    LAST_CYCLE_TIME = time_us_64();
}
