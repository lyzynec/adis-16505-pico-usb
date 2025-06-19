#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

pub const VID_PID: (u16, u16) = (0x16C0, 0x27DD);
pub const DEFAULT_BAUDRATE: u32 = 115200;

pub mod cfg;

pub use adis;
use serde::{Deserialize, Serialize};

pub use heapless::Vec;
pub use postcard::accumulator::{CobsAccumulator, FeedResult};
pub use postcard::{from_bytes_cobs, to_vec_cobs};
pub use postcard::Error as PostcardError;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Message {
    CFG(cfg::CFG),
    RQR(u16),
    B16(cfg::BurstSel, adis::burstmem::BurstMemory16),
    B32(cfg::BurstSel, adis::burstmem::BurstMemory32),
    RST,
    ERR(u8),
}
