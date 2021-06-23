use crate::{
    binding::*,
    critical::{self, CriticalSection},
    ctypes::c_void,
    cycling::{self, CycleData},
    offline::{self, BulkCycleData},
    state::{self, ProgramState},
};

use arrayvec::ArrayVec;

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
    StartSession,
    StopSession,
    Handshake { session_active: bool },
}

impl RxCommand {
    const BUF_SIZE: usize = 2 + mem::size_of::<Self>();
    const CMD_START_SESSION: u8 = 1;
    const CMD_STOP_SESSION: u8 = 2;
    const CMD_HANDSHAKE: u8 = 3;

    fn expected_len(raw: u8) -> Option<usize> {
        let data_size = match raw {
            Self::CMD_START_SESSION => Some(0),
            Self::CMD_STOP_SESSION => Some(0),
            Self::CMD_HANDSHAKE => Some(1),
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
                Self::CMD_START_SESSION => Some(Self::StartSession),
                Self::CMD_STOP_SESSION => Some(Self::StopSession),
                Self::CMD_HANDSHAKE => {
                    let flags = data[0];

                    let session_active = (flags & (1 << 0)) != 0;

                    Some(Self::Handshake { session_active })
                }
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
    const CMD_LIVE_DATA: u8 = 1;
    const CMD_BULK_DATA: u8 = 2;

    fn serialize<'a>(
        self,
        buf: &'a mut [u8; mem::size_of::<Self>()],
    ) -> Result<&'a mut [u8], Error> {
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
}

pub struct HostInterface {
    uart_dev: *mut c_void,
    tx_cmd_bufs: [ArrayVec<TxCommand, { Self::TX_CMD_BUF_SIZE }>; 2],
    cur_tx_cmd_buf: usize,
    expected_cmd_len: usize,
    cmd_receive_buffer: ArrayVec<u8, { RxCommand::BUF_SIZE }>,
    connection: Option<Connection>,
}

impl HostInterface {
    const BAUD_RATE: u32 = 9600;
    const TX_PIN: u32 = 0;
    const RX_PIN: u32 = 1;

    const TX_CMD_BUF_SIZE: usize = 64;

    const CMD_SEND_DELAY_MS: u32 = 1;

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
            tx_cmd_bufs: [ArrayVec::new(), ArrayVec::new()],
            cur_tx_cmd_buf: 0,
            expected_cmd_len: 0,
            cmd_receive_buffer: ArrayVec::new(),
            connection: None,
        });
    }

    pub fn push_cycle(&mut self, cs: &CriticalSection, data: CycleData) -> Result<(), Error> {
        let mut connection = self.connection.take();

        let result = match &mut connection {
            Some(Connection {
                started: true,
                ..
            }) => {
                // in the rare event that the session cannot hold any more cycles,
                // discard all cycles that do not fit.
                // the cycles will still be sent over bluetooth
                self.queue_cmd(cs, TxCommand::LiveData(data))?;
                Ok(())
            }
            Some(Connection { started: false, .. }) => Err(Error::NotStarted),
            None => Err(Error::NoConnection),
        };

        self.connection = connection;
        result
    }

    fn queue_cmd(&mut self, _: &CriticalSection, cmd: TxCommand) -> Result<(), Error> {
        self.tx_cmd_bufs[self.cur_tx_cmd_buf]
            .try_push(cmd)
            .map_err(|_| Error::BufferFull)
    }

    pub fn update(&mut self) {
        if let Some(Connection {
            connection_lost: false,
            ..
        }) = self.connection
        {
            // send any pending cmds from the buffer not currently being written to in interrupts
            let last_buf = (self.cur_tx_cmd_buf + 1) % 2;

            for cmd in self.tx_cmd_bufs[last_buf].drain(..) {
                let mut buf = [0u8; mem::size_of::<TxCommand>()];
                let used = cmd.serialize(&mut buf).unwrap();

                unsafe {
                    binding_uart_write_blocking(self.uart_dev, used.as_ptr(), used.len() as u32);
                    sleep_ms(Self::CMD_SEND_DELAY_MS);
                }
            }

            critical::run(|_| self.cur_tx_cmd_buf = last_buf);
        }

        // do nothing if not connected, generated commands will accumulate in the buffer
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
                    let hue = if connection.started {
                        STARTED_HUE
                    } else {
						CONNECTED_HUE
                    };

                    state::store(
                        cs,
                        ProgramState::Running {
                            status_hue: hue,
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
                    for i in 0..2 {
                        self.tx_cmd_bufs[i].clear();
                    }
                    self.start_online(cs);
                }
            }
        }
    }

    fn start_online(&mut self, cs: &CriticalSection) {
        self.connection = Some(Connection {
            connection_lost: false,
            started: false,
        });

        self.enable_uart_rx_interrupt();
        state::store(
            cs,
            ProgramState::Running {
                status_hue: CONNECTED_HUE,
            },
        );
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
            RxCommand::StartSession => self.cmd_start_session(cs),
            RxCommand::StopSession => self.cmd_stop_session(cs),
            RxCommand::Handshake { session_active } => self.cmd_handshake(cs, session_active),
        }
    }

    fn cmd_handshake(&mut self, cs: &CriticalSection, session_active: bool) {
        if let Some(Connection {
            started: false,
            connection_lost: false,
        }) = self.connection
        {
            if session_active {
                if let Some(session) = offline::take_session(cs) {
                    self.queue_cmd(cs, TxCommand::BulkData(session)).unwrap();
                }
                self.cmd_start_session(cs);
            }
        }
    }

    fn cmd_start_session(&mut self, cs: &CriticalSection) {
        cycling::reset(cs);

        self.connection = Some(Connection {
            started: true,
            connection_lost: false,
        });

        state::store(
            cs,
            ProgramState::Running {
                status_hue: STARTED_HUE,
            },
        );
    }

    fn cmd_stop_session(&mut self, _cs: &CriticalSection) {
        // todo!()
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
            if interface.cmd_receive_buffer.is_empty() {
                if let Some(expected) = RxCommand::expected_len(byte) {
                    interface.expected_cmd_len = expected;
                } else {
                    continue;
                }
            }

            // record the current command...
            interface.cmd_receive_buffer.push(byte);

            // ... until we got the expected amount of bytes
            if interface.cmd_receive_buffer.len() == interface.expected_cmd_len {
                // and try to deserialize it
                if let Some(cmd) = RxCommand::deserialize(&interface.cmd_receive_buffer) {
                    interface.execute_rx_cmd(cs, cmd);
                }

                // ready to receive a new command
                interface.cmd_receive_buffer.clear();
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
                interface.disable_uart_rx_interrupt();
                interface.connection = None;
                offline::start(cs);
            }
        }
        hardware_alarm_unclaim(CONNECTION_ALARM_NUM);
    }
}
