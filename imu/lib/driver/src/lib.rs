pub use protocol;
use serialport5 as serialport;
use std::io::{Read, Write};
use std::time::{SystemTime, SystemTimeError};
pub use std::time::Duration;

use heapless;
use thiserror::Error;

pub use protocol::adis::version::AdisVersion;

const MAX_MESSAGE_LEN: usize = 256;


#[derive(Debug, Error)]
pub enum AdisDeviceError {
    #[error("There was error in serialport: {0}.")]
    PortError(#[from] serialport::Error),
    #[error("No suitable port has been found.")]
    NoPort,
    #[error("There was IO error: {0}.")]
    IoError(#[from] std::io::Error),
    #[error("There was serialization error: {0}.")]
    SerializationError(protocol::PostcardError),
    #[error("There was no response to query.")]
    NoResponse,
    #[error("There was system time error: {0}.")]
    TimeError(#[from] SystemTimeError),
    #[error("Device sends error with tag: {0}.")]
    DeviceError(u8),
    #[error("Unspecified error occurred.")]
    Other,
}

type AdisDeviceResult<T> = Result<T, AdisDeviceError>;

pub struct AdisDevice {
    port: serialport::SerialPort,
    buffer: protocol::CobsAccumulator<MAX_MESSAGE_LEN>,
    version: protocol::adis::version::AdisVersion,
}

impl AdisDevice {
    pub fn from_device_name<S: Into<String>, B: Into<u32>>(
        path: S,
        baud_rate: B,
        version: AdisVersion,
        timeout: Option<Duration>,
    ) -> AdisDeviceResult<Self> {
        let port: serialport::SerialPort = serialport::SerialPort::builder()
            .baud_rate(baud_rate.into())
            .read_timeout(timeout)
            .open(path.into())?;

        Ok(Self {
            port,
            buffer: protocol::CobsAccumulator::new(),
            version,
        })
    }

    pub fn from_vid_pid(
        vid: u16,
        pid: u16,
        baud_rate: u32,
        version: AdisVersion,
        timeout: Option<Duration>,
    ) -> AdisDeviceResult<Self> {
        let port_name =
            serialport::available_ports()?
                .iter()
                .find_map(|p| match p.port_type.clone() {
                    serialport::SerialPortType::UsbPort(info) => {
                        if (info.vid, info.pid) == (vid, pid) {
                            Some(p.port_name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                });

        return if let Some(path) = port_name {
            Self::from_device_name(path, baud_rate, version, timeout)
        } else {
            Err(AdisDeviceError::NoPort)
        };
    }
}

impl AdisDevice {
    pub fn receive(&mut self) -> AdisDeviceResult<heapless::Vec<protocol::Message, 8>> {
        let bytes_to_read = self.port.bytes_to_read()?;
        let bytes_to_read = std::cmp::min(bytes_to_read as usize, MAX_MESSAGE_LEN);

        if bytes_to_read == 0 {
            return Ok(heapless::Vec::new());
        };

        let mut read_buffer = [0; MAX_MESSAGE_LEN];
        let read_bytes = self.port.read(&mut read_buffer[..bytes_to_read])?;

        let mut window = &read_buffer[..read_bytes];

        let mut out = heapless::Vec::new();

        'cobs: while !window.is_empty() {
            window = match self.buffer.feed::<protocol::Message>(&window) {
                protocol::FeedResult::Consumed => break 'cobs,
                protocol::FeedResult::OverFull(new_wind) => new_wind,
                protocol::FeedResult::DeserError(new_wind) => new_wind,
                protocol::FeedResult::Success { data, remaining } => {
                    if let protocol::Message::ERR(tag) = data {
                        return Err(AdisDeviceError::DeviceError(tag));
                    }

                    out.push(data).ok();

                    remaining
                }
            };
        }

        return Ok(out);
    }

    pub fn send(&mut self, message: &protocol::Message) -> AdisDeviceResult<()> {
        let write_buffer: heapless::Vec<_, MAX_MESSAGE_LEN> =
            protocol::to_vec_cobs(message).map_err(|e| AdisDeviceError::SerializationError(e))?;
            
        self.port.write(&write_buffer)?;

        return Ok(());
    }
}

impl AdisDevice {
    pub fn confirmed_send(
        &mut self,
        message: &protocol::Message,
        response_timeout: Option<Duration>,
    ) -> AdisDeviceResult<()> {
        self.send(&message)?;
        let start_time = SystemTime::now();

        while response_timeout.is_none()
            || start_time.elapsed()? < unsafe { response_timeout.unwrap_unchecked() }
        {
            let received_messages = self.receive()?;
            for m in received_messages {
                if m == *message {
                    return Ok(());
                }
            }
        }

        return Err(AdisDeviceError::NoResponse);
    }

    pub fn send_request_response(
        &mut self,
        request: u16,
        response_timeout: Option<Duration>,
    ) -> AdisDeviceResult<u16> {
        self.send(&protocol::Message::RQR(request))?;
        let start_time = SystemTime::now();

        while response_timeout.is_none()
            || start_time.elapsed()? < unsafe { response_timeout.unwrap_unchecked() }
        {
            let received_messages = self.receive()?;
            for m in received_messages {
                match m {
                    protocol::Message::RQR(response) => return Ok(response),
                    _ => (),
                }
            }
        }

        return Err(AdisDeviceError::NoResponse);
    }

    pub fn send_restart(&mut self) -> AdisDeviceResult<()> {
        return self.confirmed_send(&protocol::Message::RST, Some(Duration::from_millis(1)));
    }

    pub fn send_config(&mut self, config: protocol::cfg::CFG) -> AdisDeviceResult<()> {
        return self.confirmed_send(
            &protocol::Message::CFG(config),
            Some(Duration::from_millis(1)),
        );
    }

    pub fn send_error(&mut self, tag: u8) -> AdisDeviceResult<()> {
        return self.confirmed_send(&protocol::Message::ERR(tag), Some(Duration::from_millis(1)));
    }

    pub fn expect_burst(
        &mut self,
    ) -> AdisDeviceResult<heapless::Vec<protocol::adis::BurstData, 8>> {
        let mut out = heapless::Vec::new();
        let received_messages = self.receive()?;

        received_messages.iter().for_each(|m| match m {
            protocol::Message::B16(sel, burst) => {
                let new_out = match sel {
                    protocol::cfg::BurstSel::Sel0 => protocol::adis::BurstData::as_sel0(burst, &self.version),
                    protocol::cfg::BurstSel::Sel1 => protocol::adis::BurstData::as_sel1(burst, &self.version),
                };
                out.push(new_out).ok();
            }

            protocol::Message::B32(sel, burst) => {
                let new_out = match sel {
                    protocol::cfg::BurstSel::Sel0 => protocol::adis::BurstData::as_sel0(burst, &self.version),
                    protocol::cfg::BurstSel::Sel1 => protocol::adis::BurstData::as_sel1(burst, &self.version),
                };
                out.push(new_out).ok();
            }
            _ => (),
        });

        return Ok(out);
    }
}
