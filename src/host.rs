use crate::{
    binding::*,
    ctypes::c_void,
    cycling::CycleData,
};

static mut INSTANCE_CREATED: bool = false;

#[derive(Clone, Copy)]
pub enum OperatingMode {
    Offline,
    Online,
}

#[derive(Debug)]
pub enum Error {
    PostcardError(postcard::Error),
    NotRunning,
}

impl From<postcard::Error> for Error {
    fn from(val: postcard::Error) -> Self {
        Self::PostcardError(val)
    }
}

pub struct HostInterface {
    uart_dev: *mut c_void,
    operating_mode: Option<OperatingMode>,
    lost_connection_buf: [CycleData; Self::LOST_CONNECTION_BUF_SIZE],
    lost_connection_item_count: usize,
    connection_lost_time: Option<u64>,
}

impl HostInterface {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    const STATE_PIN: u32 = 21;

    const LOST_CONNECTION_BUF_SIZE: usize = 64;

    /// Creates the interface to the host, returning None if one is already active
    /// # Safety
    /// This function is not thread safe as it uses non-atomic operations to check whether an interface is already active.
    pub unsafe fn new() -> Option<Self> {
        if INSTANCE_CREATED {
            return None;
        }

        let uart_dev = binding_uart0_init(Self::BAUD_RATE, Self::TX_PIN, Self::RX_PIN);

        INSTANCE_CREATED = true;

        // set device name
        execute_cmd(uart_dev, b"AT+NAME=GoCycling");

        // turn off onboard led
        execute_cmd(uart_dev, b"AT+LED2M=1");

        binding_gpio_set_dir(Self::STATE_PIN, false);

        Some(Self {
            uart_dev,
            operating_mode: None,
            lost_connection_buf: [CycleData::default(); Self::LOST_CONNECTION_BUF_SIZE],
            lost_connection_item_count: 0,
            connection_lost_time: None,
        })
    }

    pub fn push(&mut self, data: &CycleData) -> Result<(), Error> {
        match self.operating_mode.ok_or(Error::NotRunning)? {
            OperatingMode::Offline => todo!(),
            OperatingMode::Online => unsafe {
                // check if bluetooth is still connected
                if binding_gpio_get(Self::STATE_PIN) {
                    // high if not connected

                    if self.lost_connection_item_count < Self::LOST_CONNECTION_BUF_SIZE {
                        self.lost_connection_buf[self.lost_connection_item_count] = *data;
                        self.lost_connection_item_count += 1;
                    }
                    else {
                        // TODO: stop the session, allow starting a new session later
                        // todo!();
                    }
                }
                else {
                    // send any cycles that were generated while the connection was lost
                    for item in &self.lost_connection_buf[0..self.lost_connection_item_count] {
                        self.send_data(item)?;
                    }
                    self.lost_connection_item_count = 0;

                    self.send_data(data)?;
                }

                Ok(())
            } 
        }
    }

    pub fn start(&mut self, mode: OperatingMode) {
        self.operating_mode = Some(mode);
    }

    unsafe fn send_data(&self, data: &CycleData) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let (buf_crc, buf_data) = buf.split_at_mut(1);
        let used = postcard::to_slice(data, buf_data)?;

        buf_crc[0] = calc_crc8(used);
        let len = 1 + used.len();

        binding_uart_write_blocking(
            self.uart_dev,
            buf[0..len].as_ptr(),
            len as u32,
        );

        Ok(())
    }
}

impl Drop for HostInterface {
    fn drop(&mut self) {
        unsafe { 
            binding_uart_destroy(self.uart_dev);
            INSTANCE_CREATED = false;
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

unsafe fn execute_cmd<const S: usize>(uart_dev: *mut c_void, cmd: &[u8; S]) {
    binding_uart_write_blocking(uart_dev, cmd.as_ptr(), cmd.len() as u32);
    // response for command AT+XXXX is always OK+XXXX so expect a response the same size
    // as the command sent
    let mut buf = [0u8; S];
    binding_uart_read_blocking(uart_dev, buf.as_mut_ptr(), buf.len() as u32);
}
