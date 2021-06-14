use crate::critical::CriticalSection;
use core::cell::UnsafeCell;

static STATE: StateWrapper = StateWrapper(UnsafeCell::new(ProgramState::WaitForModeSelect { hue: 0 }));

#[derive(Clone, Copy)]
pub enum ProgramState {
    WaitForModeSelect { hue: u8 },
    Running { status_hue: u8 },
}

/// Get a copy of the current program state
pub fn retrieve(_cs: &CriticalSection) -> ProgramState {
    unsafe {
        *STATE.0.get()
    }
}

/// Store a new program state, to be used by the next execution
pub fn store(_cs: &CriticalSection, state: ProgramState) {
    unsafe {
        *STATE.0.get() = state;
    }
}

struct StateWrapper(UnsafeCell<ProgramState>);

/// We can implement this because it's a single threaded environment and can only 
/// be accessed publically through the store/retrieve functions
unsafe impl Sync for StateWrapper {}
