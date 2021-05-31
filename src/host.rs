use crate::{
    binding::*,
    ctypes::c_void,
    cycling::CycleData,
};
use core::convert::TryFrom;

pub struct Connection {
    uart_dev: *mut c_void,
}

impl Connection {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    /// Creates the connection, returning None if a connection was already created
    pub fn new() -> Option<Self> {
        let uart_dev = unsafe { binding_uart0_init(Self::BAUD_RATE, Self::TX_PIN, Self::RX_PIN) };
        
        Some(Connection {
            uart_dev,
        })
    }

    pub fn push(&self, data: &CycleData) -> Result<(), postcard::Error> {
        let mut buf = [0u8; 32];
        let used = postcard::to_slice(data, &mut buf)?;

        unsafe { binding_uart_write_blocking(self.uart_dev, used.as_ptr(), u32::try_from(used.len()).unwrap()) }

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { 
            binding_uart_destroy(self.uart_dev);
        }
    }
}

