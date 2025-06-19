#[duplicate::duplicate_item(
    name            address;
    [DIAG_STAT]     [0x00];

    [X_GYRO_LOW]    [0x04];
    [X_GYRO_OUT]    [0x06];
    [Y_GYRO_LOW]    [0x08];
    [Y_GYRO_OUT]    [0x0A];
    [Z_GYRO_LOW]    [0x0C];
    [Z_GYRO_OUT]    [0x0E];

    [X_ACCL_LOW]    [0x10];
    [X_ACCL_OUT]    [0x12];
    [Y_ACCL_LOW]    [0x14];
    [Y_ACCL_OUT]    [0x16];
    [Z_ACCL_LOW]    [0x18];
    [Z_ACCL_OUT]    [0x1A];

    [TEMP_OUT]      [0x1C];
    [TIME_STAMP]    [0x1E];

    [DATA_CNTR]     [0x22];

    [X_DELTANG_LOW] [0x24];
    [X_DELTANG_OUT] [0x26];
    [Y_DELTANG_LOW] [0x28];
    [Y_DELTANG_OUT] [0x2A];
    [Z_DELTANG_LOW] [0x2C];
    [Z_DELTANG_OUT] [0x2E];

    [X_DELTVEL_LOW] [0x30];
    [X_DELTVEL_OUT] [0x32];
    [Y_DELTVEL_LOW] [0x34];
    [Y_DELTVEL_OUT] [0x36];
    [Z_DELTVEL_LOW] [0x38];
    [Z_DELTVEL_OUT] [0x3A];

    [XG_BIAS_LOW]   [0x40];
    [XG_BIAS_HIGH]  [0x42];
    [YG_BIAS_LOW]   [0x44];
    [YG_BIAS_HIGH]  [0x46];
    [ZG_BIAS_LOW]   [0x48];
    [ZG_BIAS_HIGH]  [0x4A];

    [XA_BIAS_LOW]   [0x4C];
    [XA_BIAS_HIGH]  [0x4E];
    [YA_BIAS_LOW]   [0x50];
    [YA_BIAS_HIGH]  [0x52];
    [ZA_BIAS_LOW]   [0x54];
    [ZA_BIAS_HIGH]  [0x56];

    [FILT_CTRL]     [0x5C];
    [RANG_MDL]      [0x5E];
    [MSC_CTRL]      [0x60];
    [UP_SCALE]      [0x62];
    [DEC_RATE]      [0x64];

    [GLOB_CMD]      [0x68];

    [FIRM_REV]      [0x6C];
    [FIRM_DM]       [0x6E];
    [FIRM_Y]        [0x70];
    [PROD_ID]       [0x72];
    [SERIAL_NUM]    [0x74];
    [USER_SCR1]     [0x76];
    [USER_SCR2]     [0x78];
    [USER_SCR3]     [0x7A];
    [FLSHCNT_LOW]   [0x7C];
    [FLSHCNT_HIGH]  [0x7E];
)]
pub const name: u8 = address;

pub const fn request(address: u8) -> u16 {
    return (address as u16) << 8;
}

pub const fn to_write(address: u8, data: u16) -> [u16; 2] {
    let u0 = (address as u16) << 8 | (data & 0xFF) | 0x8000;
    let u1 = ((address as u16) + 1) << 8 | (data >> 8) | 0x8000;
    return [u0, u1];
}
