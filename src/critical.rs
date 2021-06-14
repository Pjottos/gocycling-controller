use crate::binding::*;

use core::marker::PhantomData;

pub struct CriticalSection<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> CriticalSection<'a> {
    pub unsafe fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

pub fn run<F, R>(closure: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe {
        let status = binding_save_and_disable_interrupts();
        let result = closure(&CriticalSection::new());
        binding_restore_interrupts(status);

        result
    }
}

