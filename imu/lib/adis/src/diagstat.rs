#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod offset {
    pub const ACCL_FAIL: usize = 10;
    pub const GYRO2_FAIL: usize = 9;
    pub const GYRO1_FAIL: usize = 8;
    pub const CLOCK_ERR: usize = 7;
    pub const FLASH_ERR: usize = 6;
    pub const SENSOR_FAIL: usize = 5;
    pub const STANDBY_MODE: usize = 4;
    pub const SPI_ERR: usize = 3;
    pub const FLASH_UPDATE_ERR: usize = 2;
    pub const DATA_PATH_OVERRUN: usize = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DiagStat {
    pub accl_fail: bool,
    pub gyro2_fail: bool,
    pub gyro1_fail: bool,
    pub clock_err: bool,
    pub flash_err: bool,
    pub sensor_fail: bool,
    pub standby_mode: bool,
    pub spi_err: bool,
    pub flash_update_err: bool,
    pub data_path_overrun: bool,
}

impl From<u16> for DiagStat {
    fn from(val: u16) -> Self {
        return Self {
            accl_fail: val & (1 << offset::ACCL_FAIL) != 0,
            gyro2_fail: val & (1 << offset::GYRO2_FAIL) != 0,
            gyro1_fail: val & (1 << offset::GYRO1_FAIL) != 0,
            clock_err: val & (1 << offset::CLOCK_ERR) != 0,
            flash_err: val & (1 << offset::FLASH_ERR) != 0,
            sensor_fail: val & (1 << offset::SENSOR_FAIL) != 0,
            standby_mode: val & (1 << offset::STANDBY_MODE) != 0,
            spi_err: val & (1 << offset::SPI_ERR) != 0,
            flash_update_err: val & (1 << offset::FLASH_UPDATE_ERR) != 0,
            data_path_overrun: val & (1 << offset::DATA_PATH_OVERRUN) != 0,
        };
    }
}

impl Into<u16> for DiagStat {
    fn into(self) -> u16 {
        let mut val = 0;
        val |= if self.accl_fail { 1 } else { 0 } << offset::ACCL_FAIL;
        val |= if self.gyro2_fail { 1 } else { 0 } << offset::GYRO2_FAIL;
        val |= if self.gyro1_fail { 1 } else { 0 } << offset::GYRO1_FAIL;
        val |= if self.clock_err { 1 } else { 0 } << offset::CLOCK_ERR;
        val |= if self.flash_err { 1 } else { 0 } << offset::FLASH_ERR;
        val |= if self.sensor_fail { 1 } else { 0 } << offset::SENSOR_FAIL;
        val |= if self.standby_mode { 1 } else { 0 } << offset::STANDBY_MODE;
        val |= if self.spi_err { 1 } else { 0 } << offset::SPI_ERR;
        val |= if self.flash_update_err { 1 } else { 0 } << offset::FLASH_UPDATE_ERR;
        val |= if self.data_path_overrun { 1 } else { 0 } << offset::DATA_PATH_OVERRUN;
        return val;
    }
}
