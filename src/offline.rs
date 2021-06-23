use crate::{
    binding::*,
    critical::{self, CriticalSection},
    cycling::{self, CycleData},
    state::{self, ProgramState},
};
use serde::Serialize;

static mut CURRENT_BULK: Option<BulkCycleData> = None;

const OFFLINE_MODE_HUE: u8 = 190;

pub enum Error {
    BulkFull,
    NotActive,
}

#[derive(Serialize, Clone, Copy)]
pub struct BulkCycleData {
    millis: u32,
    cycle_count: u16,
    // session_flags: SessionFlags,
}

impl BulkCycleData {
    pub const fn new() -> Self {
        Self {
            millis: 0,
            cycle_count: 0,
            // session_flags: SessionFlags::empty(),
        }
    }

    pub fn add_cycle(&mut self, data: &CycleData) -> Result<(), Error> {
        let millis_result = self.millis.overflowing_add(data.millis);

        if self.cycle_count == u16::MAX || millis_result.1 {
            return Err(Error::BulkFull);
        }

        self.cycle_count += 1;
        self.millis = millis_result.0;

        Ok(())
    }
}

bitflags! {
    #[derive(Serialize)]
    struct SessionFlags: u8 {
        const STARTED_ONLINE = 1 << 0;
        const CLOSE_SESSION = 1 << 1;
    }
}

pub fn add_cycle(cs: &CriticalSection, data: &CycleData) -> Result<(), Error> {
    if let Some(bulk) = unsafe { CURRENT_BULK.as_mut() } {
        bulk.add_cycle(data)?;
        Ok(())
    } else {
        Err(Error::NotActive)
    }
}

pub fn start(cs: &CriticalSection) {
    unsafe {
        CURRENT_BULK = Some(BulkCycleData::new());
    }

    cycling::reset(cs);

    state::store(
        cs,
        ProgramState::Running {
            status_hue: OFFLINE_MODE_HUE,
        },
    );
}

pub fn take_session(_: &CriticalSection) -> Option<BulkCycleData> {
    unsafe { CURRENT_BULK.take() }
}
