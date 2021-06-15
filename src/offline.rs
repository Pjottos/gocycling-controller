use crate::{binding::*, cycling::CycleData};

pub struct BulkCycleData {
    cycle_count: u16,
    micros: u32,
}

impl BulkCycleData {
    pub fn new() -> Self {
        Self {
            cycle_count: 0,
            micros: 0,
        }
    }

    pub fn add_cycle(&mut self, data: CycleData) {
        self.cycle_count += 1;
        self.micros += data.micros;
    }
}
