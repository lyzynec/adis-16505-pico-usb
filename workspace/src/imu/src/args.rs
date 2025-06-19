use driver;
use driver::protocol;
use driver::protocol::adis;

pub const VID: u16 = protocol::VID_PID.0;
pub const PID: u16 = protocol::VID_PID.1;

pub const TOPIC_NAME_IMU: &str = "imu";
pub const TOPIC_NAME_TEMP: &str = "temp";

pub const CGF_BURST_MODE: protocol::cfg::Burst32 = protocol::cfg::Burst32::Disabled;
pub const CGF_BURST_SEL: protocol::cfg::BurstSel = protocol::cfg::BurstSel::Sel0;

pub const BAUD_RATE: u32 = protocol::DEFAULT_BAUDRATE;

pub const VERSION: adis::version::AdisVersion = adis::version::AdisVersion::ADIS16505_1BMLZ;

