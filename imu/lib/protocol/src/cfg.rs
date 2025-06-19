use serde::{Serialize, Deserialize };

use adis;

pub use adis::msc_ctrl::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CFG {
    BurstEn (bool),
    Burst32 (Burst32),
    BurstSel (BurstSel),
    LinearAccelerationCompensation (LinearAccelerationCompensation),
    PointOfPercussionAlignment (PointOfPercussionAlignment),
    SensorBandwidth (SensorBandwidth),
    SyncPolarity (SyncPolarity),
    DataReadyPolarity (DataReadyPolarity),
}
