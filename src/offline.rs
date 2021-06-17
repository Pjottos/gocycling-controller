use crate::{
    binding::*,
    critical::{self, CriticalSection},
    cycling::CycleData,
    state::{self, ProgramState},
};
use serde::Serialize;

static mut CURRENT_BULK: BulkCycleData = BulkCycleData::new();

const OFFLINE_MODE_HUE: u8 = 220;

pub struct BulkFull;

#[derive(Serialize)]
pub struct BulkCycleData {
    millis: u32,
    cycle_count: u16,
}

impl BulkCycleData {
    pub const fn new() -> Self {
        Self {
            millis: 0,
            cycle_count: 0,
        }
    }

    pub fn add_cycle(&mut self, data: &CycleData) -> Result<(), BulkFull> {
        let millis_result = self.millis.overflowing_add(data.micros / 1000);

        if self.cycle_count == u16::MAX || millis_result.1 {
            return Err(BulkFull);
        }

        self.cycle_count += 1;
        self.millis = millis_result.0;

        Ok(())
    }
}

pub fn update(_: &CriticalSection) {
    // TODO:
}

pub fn add_cycle(cs: &CriticalSection, data: &CycleData) {
    unsafe {
        if CURRENT_BULK.add_cycle(data).is_err() {
            save_session_and_start_new(cs);
            // impossible to recurse more than once because start_new_session resets all fields
            add_cycle(cs, data);
        }
    }
}

pub fn save_session_and_start_new(_: &CriticalSection) {
    todo!()
}

pub fn continue_session(cs: &CriticalSection, session: BulkCycleData) {
    state::store(cs, ProgramState::Running { status_hue: OFFLINE_MODE_HUE });
}

