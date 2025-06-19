#![cfg_attr(all(not(test), not(feature = "cburst")), no_std)]

pub use uom;
pub use uom::si::f64::{Acceleration, Angle, AngularVelocity, Velocity, ThermodynamicTemperature};

pub use uom::si::{
    acceleration::meter_per_second_squared,
    angle::{degree, radian},
    angular_velocity::{degree_per_second, radian_per_second},
    thermodynamic_temperature::degree_celsius,
    velocity::meter_per_second,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod burstmem;
pub mod diagstat;
pub mod memorymap;
pub mod msc_ctrl;
pub mod version;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BurstData {
    pub diagstat: diagstat::DiagStat,
    pub data: Sel,
    pub temp: ThermodynamicTemperature,
    pub data_cntr: u16,
    pub corrupted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Sel {
    Sel0 {
        x_gyro: AngularVelocity,
        y_gyro: AngularVelocity,
        z_gyro: AngularVelocity,

        x_accl: Acceleration,
        y_accl: Acceleration,
        z_accl: Acceleration,
    },
    Sel1 {
        x_deltang: Angle,
        y_deltang: Angle,
        z_deltang: Angle,

        x_deltvel: Velocity,
        y_deltvel: Velocity,
        z_deltvel: Velocity,
    },
}

impl BurstData {
    pub fn as_sel0<T>(burst_mem: &T, version: &version::AdisVersion) -> Self
    where
        T: burstmem::BurstMemory,
    {
        return Self {
            diagstat: burst_mem.diag_stat().into(),
            data: Sel::Sel0 {
                x_gyro: AngularVelocity::new::<degree_per_second>(burst_mem.xa() * version.gyro_constant()),
                y_gyro: AngularVelocity::new::<degree_per_second>(burst_mem.ya() * version.gyro_constant()),
                z_gyro: AngularVelocity::new::<degree_per_second>(burst_mem.za() * version.gyro_constant()),
                x_accl: Acceleration::new::<meter_per_second_squared>(burst_mem.xb() * version.accl_constant()),
                y_accl: Acceleration::new::<meter_per_second_squared>(burst_mem.yb() * version.accl_constant()),
                z_accl: Acceleration::new::<meter_per_second_squared>(burst_mem.zb() * version.accl_constant()),
            },
            temp: ThermodynamicTemperature::new::<degree_celsius>(burst_mem.temp() * version.temp_constant()),
            data_cntr: burst_mem.data_cntr(),
            corrupted: burst_mem.is_corrupted(),
        };
    }

    pub fn as_sel1<T>(burst_mem: &T, version: &version::AdisVersion) -> Self
    where
        T: burstmem::BurstMemory,
    {
        return Self {
            diagstat: burst_mem.diag_stat().into(),
            data: Sel::Sel1 {
                x_deltang: Angle::new::<degree>(burst_mem.xa() * version.deltang_constant()),
                y_deltang: Angle::new::<degree>(burst_mem.ya() * version.deltang_constant()),
                z_deltang: Angle::new::<degree>(burst_mem.za() * version.deltang_constant()),
                x_deltvel: Velocity::new::<meter_per_second>(burst_mem.xb() * version.deltvel_constant()),
                y_deltvel: Velocity::new::<meter_per_second>(burst_mem.yb() * version.deltvel_constant()),
                z_deltvel: Velocity::new::<meter_per_second>(burst_mem.zb() * version.deltvel_constant()),
            },
            temp: ThermodynamicTemperature::new::<degree_celsius>(burst_mem.temp() * version.temp_constant()),
            data_cntr: burst_mem.data_cntr(),
            corrupted: burst_mem.is_corrupted(),
        };
    }
}

#[cfg(feature = "cburst")]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CBurstData {
    pub diagstat: u16,
    pub sel: u8,
    pub xa: std::ffi::c_double,
    pub ya: std::ffi::c_double,
    pub za: std::ffi::c_double,
    pub xb: std::ffi::c_double,
    pub yb: std::ffi::c_double,
    pub zb: std::ffi::c_double,
    pub temp: std::ffi::c_double,
    pub data_cntr: u16,
    pub corrupted: u8,
}

#[cfg(feature = "cburst")]
impl From<&BurstData> for CBurstData {
    fn from(burst_data: &BurstData) -> Self {
        let (sel, xa, ya, za, xb, yb, zb) = match burst_data.data {
            Sel::Sel0 {
                x_gyro,
                y_gyro,
                z_gyro,
                x_accl,
                y_accl,
                z_accl,
            } => (0,
                x_gyro.get::<radian_per_second>(),
                y_gyro.get::<radian_per_second>(),
                z_gyro.get::<radian_per_second>(),
                x_accl.get::<meter_per_second_squared>(),
                y_accl.get::<meter_per_second_squared>(),
                z_accl.get::<meter_per_second_squared>()
            ),
            Sel::Sel1 {
                x_deltang,
                y_deltang,
                z_deltang,
                x_deltvel,
                y_deltvel,
                z_deltvel,
            } => (
                1,
                x_deltang.get::<radian>(),
                y_deltang.get::<radian>(),
                z_deltang.get::<radian>(),
                x_deltvel.get::<meter_per_second>(),
                y_deltvel.get::<meter_per_second>(),
                z_deltvel.get::<meter_per_second>(),
            ),
        };

        return Self {
            diagstat: burst_data.diagstat.into(),
            sel,
            xa,
            ya,
            za,
            xb,
            yb,
            zb,
            temp: burst_data.temp.get::<degree_celsius>(),
            data_cntr: burst_data.data_cntr,
            corrupted: burst_data.corrupted as u8,
        };
    }
}
