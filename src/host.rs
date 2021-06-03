use crate::{
    binding::*,
    ctypes::c_void,
    cycling::CycleData,
};

static mut UART_CREATED: bool = false;

pub struct Connection {
    uart_dev: *mut c_void,
}

impl Connection {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    /// Creates the connection, returning None if a connection was already created
    /// # Safety
    /// This function is not thread safe as it uses non-atomic operations to check whether a device was already created.
    pub unsafe fn new() -> Option<Self> {
        if UART_CREATED {
            return None;
        }

        let uart_dev = binding_uart0_init(Self::BAUD_RATE, Self::TX_PIN, Self::RX_PIN);

        UART_CREATED = true;
        
        Some(Connection {
            uart_dev,
        })
    }

    pub fn push(&self, data: &CycleData) -> Result<(), postcard::Error> {
        let mut buf = [0u8; 20];
        let (buf_crc, buf_data) = buf.split_at_mut(1);
        let used = postcard::to_slice(data, buf_data)?;

        buf_crc[0] = calc_crc8(used);
        let len = 1 + used.len();
        unsafe { binding_uart_write_blocking(
            self.uart_dev,
            buf[0..len].as_ptr(),
            len as u32,
        ); }

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { 
            binding_uart_destroy(self.uart_dev);
            UART_CREATED = false;
        }
    }
}


fn calc_crc8(data: &[u8]) -> u8 {
    let mut crc = 0xFF;

    for val in data.iter().copied() {
        crc ^= val;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ 0x31;
            }
            else {
                crc <<= 1;
            }
        }
    }

    crc
}

