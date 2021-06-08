use crate::{
    binding::*,
    ctypes::c_void,
    cycling::CycleData,
};

pub static mut HOST_INTERFACE: Option<HostInterface> = None;

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

enum Command {
    StartSession { timestamp: datetime_t },
}

impl Command {
    const CMD_START_SESSION: u8 = 1;

    fn expected_len(raw: u8) -> Option<usize> {
        let data_size = match raw {
            Self::CMD_START_SESSION => Some(5),
            _ => None,
        };

        data_size.map(|size| 2 + size)
    }

    fn deserialize(raw: &[u8]) -> Option<Self> {
        if raw.len() == 0 {
            return None;
        }

        let data = Self::command_data(raw);

        match raw[0] {
            Self::CMD_START_SESSION => {
                if data.len() != 7 || !Self::verify_crc8(raw) {
                    None
                } else {
                    let bytes = [raw[2], raw[3], raw[4], raw[5], raw[6], 0, 0, 0];
                    let timestamp = datetime_t::from_bits(u64::from_le_bytes(bytes));
                    Some(Self::StartSession { timestamp })
                }
            },
            _ => None,
        }
    }

    fn verify_crc8(raw: &[u8]) -> bool {
        if raw.len() < 2 {
            return false;
        }

        let expected = raw[1];

        expected == calc_crc8(Self::command_data(raw))
    }

    fn command_data<'a>(raw: &'a [u8]) -> &'a [u8] {
        if raw.len() < 2 {
            &raw[..0]
        }
        else {
            &raw[2..]
        }
    }
}

pub struct HostInterface {
    uart_dev: *mut c_void,
    operating_mode: Option<OperatingMode>,
    lost_connection_buf: [CycleData; Self::LOST_CONNECTION_BUF_SIZE],
    lost_connection_item_count: usize,
    connection_lost_time: Option<u64>,
    cmd_receive_buffer: [u8; Self::MAX_COMMAND_SIZE],
    cur_cmd_len: usize,
    expected_cmd_len: usize,
}

impl HostInterface {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    pub const PIN_CONNECTION_STATE: u32 = 21;

    const LOST_CONNECTION_BUF_SIZE: usize = 64;

    const MAX_COMMAND_SIZE: usize = 64;

    pub unsafe fn create() {
        let uart_dev = binding_uart0_init(Self::BAUD_RATE, Self::TX_PIN, Self::RX_PIN);

        binding_irq_set_exclusive_handler(UART0_IRQ, Some(on_uart0_rx));
        binding_irq_set_enabled(UART0_IRQ, true);

        binding_uart_set_irq_enables(uart_dev, true, false);

        // set device name
        execute_at_cmd(uart_dev, b"AT+NAME=GoCycling");

        // turn off onboard led
        // execute_at_cmd(uart_dev, b"AT+LED2M=1");

        binding_gpio_set_dir(Self::PIN_CONNECTION_STATE, false);

        HOST_INTERFACE = Some(Self {
            uart_dev,
            operating_mode: None,
            lost_connection_buf: [CycleData::default(); Self::LOST_CONNECTION_BUF_SIZE],
            lost_connection_item_count: 0,
            connection_lost_time: None,
            cmd_receive_buffer: [0u8; Self::MAX_COMMAND_SIZE],
            cur_cmd_len: 0,
            expected_cmd_len: 0,
        });
    }

    pub fn push(&mut self, data: &CycleData) -> Result<(), Error> {
        match self.operating_mode.ok_or(Error::NotRunning)? {
            OperatingMode::Offline => todo!(),
            OperatingMode::Online => unsafe {
                // check if bluetooth is still connected
                // TODO: handle with interrupt
                if binding_gpio_get(Self::PIN_CONNECTION_STATE) {
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

    pub fn connection_changed(&mut self, connected: bool) {

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

    fn execute_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::StartSession { timestamp } => {
                // TODO
            },
        }
    }
}

impl Drop for HostInterface {
    fn drop(&mut self) {
        unsafe { 
            binding_uart_destroy(self.uart_dev);
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


unsafe fn execute_at_cmd<const S: usize>(uart_dev: *mut c_void, cmd: &[u8; S]) {
    binding_uart_write_blocking(uart_dev, cmd.as_ptr(), cmd.len() as u32);
    // response for command AT+XXXX is always OK+XXXX so expect a response the same size
    // as the command sent
    let mut buf = [0u8; S];
    binding_uart_read_blocking(uart_dev, buf.as_mut_ptr(), buf.len() as u32);
}

unsafe extern "C" fn on_uart0_rx() {
    if let Some(interface) = HOST_INTERFACE.as_mut() {
        while binding_uart_is_readable(interface.uart_dev) {
            let char = binding_uart_getc(interface.uart_dev);

            // start receiving new command, discarding the byte if it is not a valid command
            if interface.cur_cmd_len == 0 {
                if let Some(expected) = Command::expected_len(char) {
                    interface.expected_cmd_len = expected;
                }
                else {
                    continue;
                }
            }

            // record the current command...
            interface.cmd_receive_buffer[interface.cur_cmd_len] = char; 
            interface.cur_cmd_len += 1;

            // ... until we got the expected amount of bytes
            if interface.cur_cmd_len == interface.expected_cmd_len {
                // and try to deserialize it
                if let Some(cmd) = Command::deserialize(&interface.cmd_receive_buffer[..interface.cur_cmd_len]) {
                    interface.execute_cmd(cmd);
                }
            }
        }
    }
}
