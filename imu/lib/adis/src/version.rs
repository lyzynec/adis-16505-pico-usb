#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AdisVersion {
    ADIS16505_1BMLZ,
    ADIS16505_2BMLZ,
    ADIS16505_3BMLZ,

    ADIS16465_1BMLZ,
    ADIS16465_2BMLZ,
    ADIS16465_3BMLZ,
}

impl TryFrom<&str> for AdisVersion {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        return match s {
            "ADIS16505_1BMLZ" => Ok(Self::ADIS16505_1BMLZ),
            "ADIS16505_2BMLZ" => Ok(Self::ADIS16505_2BMLZ),
            "ADIS16505_3BMLZ" => Ok(Self::ADIS16505_3BMLZ),

            "ADIS16465_1BMLZ" => Ok(Self::ADIS16465_1BMLZ),
            "ADIS16465_2BMLZ" => Ok(Self::ADIS16465_2BMLZ),
            "ADIS16465_3BMLZ" => Ok(Self::ADIS16465_3BMLZ),

            _ => Err(()),
        };
    }
}

impl AdisVersion {
    pub fn from_id(prod_id: u32, version: u32) -> Result<Self, ()> {
        return match (prod_id, version) {
            (16505, 1) => Ok(Self::ADIS16505_1BMLZ),
            (16505, 2) => Ok(Self::ADIS16505_2BMLZ),
            (16505, 3) => Ok(Self::ADIS16505_3BMLZ),

            (16465, 1) => Ok(Self::ADIS16465_1BMLZ),
            (16465, 2) => Ok(Self::ADIS16465_2BMLZ),
            (16465, 3) => Ok(Self::ADIS16465_3BMLZ),
            _ => Err(()),
        };
    }
}

impl AdisVersion {
    #[inline(always)]
    pub fn gyro_constant(&self) -> f64 {
        return match self {
            Self::ADIS16505_1BMLZ => 0.00625,
            Self::ADIS16505_2BMLZ => 0.025,
            Self::ADIS16505_3BMLZ => 0.1,

            Self::ADIS16465_1BMLZ => 0.00625,
            Self::ADIS16465_2BMLZ => 0.025,
            Self::ADIS16465_3BMLZ => 0.1,
        };
    }

    #[inline(always)]
    pub fn accl_constant(&self) -> f64 {
        return match self {
            Self::ADIS16505_1BMLZ => 0.00245,
            Self::ADIS16505_2BMLZ => 0.00245,
            Self::ADIS16505_3BMLZ => 0.00245,

            Self::ADIS16465_1BMLZ => 2.4516625,
            Self::ADIS16465_2BMLZ => 2.4516625,
            Self::ADIS16465_3BMLZ => 2.4516625,
        };
    }

    #[inline(always)]
    pub fn deltang_constant(&self) -> f64 {
        return match self {
            Self::ADIS16505_1BMLZ => 360.0,
            Self::ADIS16505_2BMLZ => 720.0,
            Self::ADIS16505_3BMLZ => 2160.0,

            Self::ADIS16465_1BMLZ => 360.0,
            Self::ADIS16465_2BMLZ => 720.0,
            Self::ADIS16465_3BMLZ => 2160.0,
        };
    }

    #[inline(always)]
    pub fn deltvel_constant(&self) -> f64 {
        return match self {
            _ => 100.0_f64 / (2_u64.pow(15) as f64),
        };
    }

    #[inline(always)]
    pub const fn temp_constant(&self) -> f64 {
        return match self {
            _ => 0.1,
        };
    }
}

#[test]
#[rustfmt::skip]
fn test() {
    assert_eq!(AdisVersion::try_from("ADIS16505_1BMLZ"), Ok(AdisVersion::ADIS16505_1BMLZ));
    assert_eq!(AdisVersion::try_from("ADIS16505_2BMLZ"), Ok(AdisVersion::ADIS16505_2BMLZ));
    assert_eq!(AdisVersion::try_from("ADIS16505_3BMLZ"), Ok(AdisVersion::ADIS16505_3BMLZ));

    assert_eq!(AdisVersion::try_from("ADIS16465_1BMLZ"), Ok(AdisVersion::ADIS16465_1BMLZ));
    assert_eq!(AdisVersion::try_from("ADIS16465_2BMLZ"), Ok(AdisVersion::ADIS16465_2BMLZ));
    assert_eq!(AdisVersion::try_from("ADIS16465_3BMLZ"), Ok(AdisVersion::ADIS16465_3BMLZ));

    assert_eq!(AdisVersion::from_id(16505, 1), Ok(AdisVersion::ADIS16505_1BMLZ));
    assert_eq!(AdisVersion::from_id(16505, 2), Ok(AdisVersion::ADIS16505_2BMLZ));
    assert_eq!(AdisVersion::from_id(16505, 3), Ok(AdisVersion::ADIS16505_3BMLZ));

    assert_eq!(AdisVersion::from_id(16465, 1), Ok(AdisVersion::ADIS16465_1BMLZ));
    assert_eq!(AdisVersion::from_id(16465, 2), Ok(AdisVersion::ADIS16465_2BMLZ));
    assert_eq!(AdisVersion::from_id(16465, 3), Ok(AdisVersion::ADIS16465_3BMLZ));
}
