use chrono;
pub use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// name of the port
    pub device: Option<String>,

    /// vendor id of the port
    #[arg(long, default_value_t = super::DEFAULT_VID_PID.0)]
    pub vid: u16,

    /// product id of the port
    #[arg(long, default_value_t = super::DEFAULT_VID_PID.1)]
    pub pid: u16,

    /// baudrate to be used for port
    #[arg(long, default_value_t = super::DEFAULT_BAUDRATE)]
    pub baud_rate: u32,

    pub timeout_ns: Option<u64>,

    /// log file path
    #[arg(long, default_value_t = ("").to_string())]
    pub log_path: String,

    /// log file name
    #[arg(long, default_value_t = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string())]
    pub log_name: String,

    /// burst mode
    /// 16 - half precision
    /// 32 - full precision (not supported on all devices)
    #[arg(long, default_value_t = 16)]
    pub burst_mode: u32,

    /// burst sel
    /// 0 - selected gyro and accl data
    /// 1 - selected deltang and deltvel data
    #[arg(long, default_value_t = 0)]
    pub burst_sel: u32,

    /// device number
    #[arg(long, default_value_t = 16505)]
    pub board_id: u32,

    /// device version
    #[arg(long, default_value_t = 1)]
    pub board_version: u32,
}
