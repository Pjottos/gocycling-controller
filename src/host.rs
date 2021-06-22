use crate::{
    binding::*,
    critical::{self, CriticalSection},
    ctypes::c_void,
    cycling::{self, CycleData},
    offline::{self, BulkCycleData},
    state::{self, ProgramState},
    uf2,
};

use core::mem;

pub static mut HOST_INTERFACE: Option<HostInterface> = None;

const CONNECTION_ALARM_NUM: u32 = 1;
const RECONNECT_TIMEOUT_US: u64 = 10_000_000;

const CONNECTED_HUE: u8 = 160;
const STARTED_HUE: u8 = 130;
const RECONNECTING_HUE: u8 = 0;

#[derive(Debug)]
pub enum Error {
    PostcardError(postcard::Error),
    NotStarted,
    NoConnection,
    BufferFull,
}

impl From<postcard::Error> for Error {
    fn from(val: postcard::Error) -> Self {
        Self::PostcardError(val)
    }
}

enum RxCommand {
    StartSession { datetime: datetime_t },
    StopSession,
    ContinueSession,
    BeginFirmwareUpdate { chunk_count: usize },
}

impl RxCommand {
    const CMD_START_SESSION: u8 = 1;
    const CMD_STOP_SESSION: u8 = 2;
    const CMD_CONTINUE_SESSION: u8 = 3;
    const CMD_BEGIN_FIRMWARE_UPDATE: u8 = 5;

    fn expected_len(raw: u8) -> Option<usize> {
        let data_size = match raw {
            Self::CMD_START_SESSION => Some(5),
            Self::CMD_STOP_SESSION => Some(0),
            Self::CMD_CONTINUE_SESSION => Some(0),
            Self::CMD_BEGIN_FIRMWARE_UPDATE => Some(mem::size_of::<usize>()),
            _ => None,
        };

        data_size.map(|size| 2 + size)
    }

    fn deserialize(raw: &[u8]) -> Option<Self> {
        if raw.len() < 2 || !Self::verify_crc8(raw) {
            return None;
        }

        if let Some(expected_len) = Self::expected_len(raw[0]) {
            let data = Self::command_data(raw);

            if raw.len() != expected_len {
                return None;
            }

            match raw[0] {
                Self::CMD_START_SESSION => {
                    let bytes = [data[0], data[1], data[2], data[3], data[4], 0, 0, 0];
                    let datetime = datetime_t::from_bits(u64::from_le_bytes(bytes));

                    Some(Self::StartSession { datetime })
                }
                Self::CMD_STOP_SESSION => Some(Self::StopSession),
                Self::CMD_CONTINUE_SESSION => Some(Self::ContinueSession),
                Self::CMD_BEGIN_FIRMWARE_UPDATE => Some(Self::BeginFirmwareUpdate {
                    chunk_count: u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize,
                }),
                // we got an expected len so the cmd should be valid
                _ => unreachable!(),
            }
        } else {
            None
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
        } else {
            &raw[2..]
        }
    }
}

enum TxCommand {
    LiveData(CycleData),
    BulkData(BulkCycleData),
}

impl TxCommand {
    const MAX_CMD_SIZE: usize = 16;
    const CMD_LIVE_DATA: u8 = 1;
    const CMD_BULK_DATA: u8 = 2;

    fn serialize<'a>(self, buf: &'a mut [u8; Self::MAX_CMD_SIZE]) -> Result<&'a mut [u8], Error> {
        let (buf_header, buf_data) = buf.split_at_mut(2);

        let data_len = match self {
            Self::LiveData(data) => {
                buf_header[0] = Self::CMD_LIVE_DATA;

                let used = postcard::to_slice(&data, buf_data)?;

                used.len()
            }
            Self::BulkData(data) => {
                buf_header[0] = Self::CMD_BULK_DATA;

                let used = postcard::to_slice(&data, buf_data)?;

                used.len()
            }
        };

        buf_header[1] = calc_crc8(&buf_data[..data_len]);

        Ok(&mut buf[..2 + data_len])
    }
}

struct Connection {
    connection_lost: bool,
    started: bool,
    session: BulkCycleData,
}

pub struct HostInterface {
    uart_dev: *mut c_void,
    cycle_buf: [CycleData; Self::CYCLE_BUF_SIZE],
    cycle_item_count: usize,
    cmd_receive_buffer: [u8; Self::MAX_COMMAND_SIZE],
    cur_cmd_len: usize,
    expected_cmd_len: usize,
    connection: Option<Connection>,
}

impl HostInterface {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    const CYCLE_BUF_SIZE: usize = 64;

    const MAX_COMMAND_SIZE: usize = 64;

    pub unsafe fn create() {
        let uart_dev = binding_uart0_init(Self::BAUD_RATE, Self::TX_PIN, Self::RX_PIN);

        // set device name
        // execute_at_cmd(uart_dev, b"AT+NAME=GoCycling");
        // the module needs some time to execute the command
        // it only starts executing after the response it seems
        // sleep_ms(50);

        // turn off onboard led
        // execute_at_cmd(uart_dev, b"AT+LED2M=1");
        // sleep_ms(1000);

        HOST_INTERFACE = Some(Self {
            uart_dev,
            cycle_buf: [CycleData::default(); Self::CYCLE_BUF_SIZE],
            cycle_item_count: 0,
            cmd_receive_buffer: [0u8; Self::MAX_COMMAND_SIZE],
            cur_cmd_len: 0,
            expected_cmd_len: 0,
            connection: None,
        });
    }

    pub fn push_cycle(&mut self, _: &CriticalSection, data: CycleData) -> Result<(), Error> {
        match self.connection {
            Some(Connection { started: true, .. }) => {
                // record cycle data in a buffer that gets emptied in the main loop
                if self.cycle_item_count < Self::CYCLE_BUF_SIZE {
                    self.cycle_buf[self.cycle_item_count] = data;
                    self.cycle_item_count += 1;
                    Ok(())
                } else {
                    Err(Error::BufferFull)
                }
            }
            Some(Connection { started: false, .. }) => Err(Error::NotStarted),
            None => Err(Error::NoConnection),
        }
    }

    pub fn update(&mut self) {
        if let Some(mut connection) = self.connection.take() {
            if !connection.connection_lost && connection.started {
                // send any pending cycles
                for item in self.cycle_buf[0..self.cycle_item_count].iter().copied() {
                    // in the rare event that the session cannot hold any more cycles,
                    // discard all cycles that do not fit.
                    // the cycles will still be sent over bluetooth
                    connection.session.add_cycle(&item).ok();
                    self.send_cmd(TxCommand::LiveData(item)).unwrap();
                }

                self.cycle_item_count = 0;
            }

            self.connection = Some(connection);
        }

        // do nothing if not connected, generated cycles will accumulate in the buffer
    }

    pub fn has_connection(&self, _: &CriticalSection) -> bool {
        self.connection.is_some()
    }

    fn start_reconnecting(cs: &CriticalSection) {
        state::store(
            cs,
            ProgramState::Running {
                status_hue: RECONNECTING_HUE,
            },
        );

        unsafe {
            let connection_gone_time = absolute_time_t {
                _private_us_since_boot: time_us_64() + RECONNECT_TIMEOUT_US,
            };

            hardware_alarm_claim(CONNECTION_ALARM_NUM);
            hardware_alarm_set_callback(CONNECTION_ALARM_NUM, Some(on_connection_alarm));
            hardware_alarm_set_target(CONNECTION_ALARM_NUM, connection_gone_time);
        }
    }

    pub fn connection_changed(&mut self, cs: &CriticalSection, value: bool) {
        match self.connection.as_mut() {
            Some(connection) => {
                connection.connection_lost = !value;

                if connection.connection_lost {
                    if connection.started {
                        Self::start_reconnecting(cs);
                    } else {
                        // no session was active, return to mode select
                        state::store(cs, ProgramState::WaitForModeSelect);
                    }
                } else {
                    state::store(
                        cs,
                        ProgramState::Running {
                            status_hue: CONNECTED_HUE,
                        },
                    );

                    unsafe {
                        // cancel alarm, we successfully reconnected
                        // does nothing if alarm was not set (when a connection is made
                        // but no session is active)
                        hardware_alarm_unclaim(CONNECTION_ALARM_NUM);
                        hardware_alarm_cancel(CONNECTION_ALARM_NUM);
                    }
                }
            }
            None => {
                if value {
                    let offline_session = offline::take_session(cs);
                    self.start_online(cs, offline_session);
                }
            }
        }
    }

    fn start_online(&mut self, cs: &CriticalSection, offline_session: Option<BulkCycleData>) {
        self.connection = Some(Connection {
            connection_lost: false,
            started: false,
            session: offline_session.unwrap_or(BulkCycleData::new()),
        });

        self.enable_uart_rx_interrupt();
        state::store(
            cs,
            ProgramState::Running {
                status_hue: CONNECTED_HUE,
            },
        );
    }

    fn queue_bulk_sync(&mut self, cs: &CriticalSection, bulk: BulkCycleData) {
        todo!()
    }

    fn enable_uart_rx_interrupt(&self) {
        unsafe {
            binding_irq_set_exclusive_handler(UART0_IRQ, Some(on_uart0_rx));
            binding_irq_set_enabled(UART0_IRQ, true);

            binding_uart_set_irq_enables(self.uart_dev, true, false);
        }
    }

    fn disable_uart_rx_interrupt(&self) {
        unsafe {
            binding_irq_set_enabled(UART0_IRQ, false);
        }
    }

    fn execute_rx_cmd(&mut self, cs: &CriticalSection, cmd: RxCommand) {
        match cmd {
            RxCommand::StartSession { mut datetime } => self.cmd_start_session(cs, &mut datetime),
            RxCommand::StopSession => self.cmd_stop_session(cs),
            RxCommand::ContinueSession => self.cmd_continue_session(cs),
            RxCommand::BeginFirmwareUpdate { chunk_count } => {
                self.cmd_begin_firmware_update(cs, chunk_count)
            }
        }
    }

    fn send_cmd(&self, cmd: TxCommand) -> Result<(), Error> {
        let mut buf = [0u8; TxCommand::MAX_CMD_SIZE];
        let used = cmd.serialize(&mut buf)?;

        unsafe {
            binding_uart_write_blocking(self.uart_dev, used.as_ptr(), used.len() as u32);
        }

        Ok(())
    }

    fn cmd_start_session(&mut self, cs: &CriticalSection, datetime: &mut datetime_t) {
        if let Some(Connection {
            started,
            connection_lost: false,
            session,
        }) = self.connection.as_mut()
        {
            unsafe {
                rtc_set_datetime(datetime);
            }
            cycling::reset(cs);
            *started = true;
            *session = BulkCycleData::new();

            state::store(cs, ProgramState::Running { status_hue: STARTED_HUE });
        }
    }

    fn cmd_stop_session(&mut self, cs: &CriticalSection) {
        todo!()
    }

    fn cmd_continue_session(&mut self, cs: &CriticalSection) {
        todo!()
    }

    fn cmd_begin_firmware_update(&mut self, cs: &CriticalSection, chunk_count: usize) {
        todo!()
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
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

/// Must be called only before the global HOST_INTERFACE is started, otherwise the
/// UART interrupt will interfere
unsafe fn execute_at_cmd<const S: usize>(uart_dev: *mut c_void, cmd: &[u8; S]) {
    binding_uart_write_blocking(uart_dev, cmd.as_ptr(), cmd.len() as u32);
    // response for command AT+XXXX is always OK+XXXX so expect a response the same size
    // as the command sent
    let mut buf = [0u8; S];
    binding_uart_read_blocking(uart_dev, buf.as_mut_ptr(), buf.len() as u32);
}

unsafe extern "C" fn on_uart0_rx() {
    // inside interrupt handler
    let cs = &CriticalSection::new();

    if let Some(interface) = HOST_INTERFACE.as_mut() {
        while binding_uart_is_readable(interface.uart_dev) {
            let byte = binding_uart_getc(interface.uart_dev);

            // start receiving new command, discarding the byte if it is not a valid command
            if interface.cur_cmd_len == 0 {
                if let Some(expected) = RxCommand::expected_len(byte) {
                    interface.expected_cmd_len = expected;
                } else {
                    continue;
                }
            }

            // record the current command...
            interface.cmd_receive_buffer[interface.cur_cmd_len] = byte;
            interface.cur_cmd_len += 1;

            // ... until we got the expected amount of bytes
            if interface.cur_cmd_len == interface.expected_cmd_len {
                // and try to deserialize it
                if let Some(cmd) =
                    RxCommand::deserialize(&interface.cmd_receive_buffer[..interface.cur_cmd_len])
                {
                    interface.execute_rx_cmd(cs, cmd);
                }

                // ready to receive a new command
                interface.cur_cmd_len = 0;
            }
        }
    }
}

unsafe extern "C" fn on_connection_alarm(alarm_num: u32) {
    let cs = &CriticalSection::new();

    if alarm_num == CONNECTION_ALARM_NUM {
        if let Some(interface) = HOST_INTERFACE.as_mut() {
            if interface
                .connection
                .as_ref()
                .map(|c| c.started)
                .unwrap_or(false)
            {
                let connection = interface.connection.take().unwrap();
                interface.disable_uart_rx_interrupt();
                offline::continue_session(cs, connection.session);
            }
        }
        hardware_alarm_unclaim(CONNECTION_ALARM_NUM);
    }
}
