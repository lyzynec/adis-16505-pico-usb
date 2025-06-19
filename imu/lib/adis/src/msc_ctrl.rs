use duplicate;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const DEFAULT_VALUE: u16 = 0x00C1;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MscCtrl {
    pub burst32: Burst32,
    pub burst_sel: BurstSel,
    pub lac: LinearAccelerationCompensation,
    pub popa: PointOfPercussionAlignment,
    pub bw: SensorBandwidth,
    pub sync_mode: SyncMode,
    pub sync_pol: SyncPolarity,
    pub dr_pol: DataReadyPolarity,
}

impl Default for MscCtrl {
    fn default() -> Self {
        return DEFAULT_VALUE.into();
    }
}

impl From<u16> for MscCtrl {
    fn from(data: u16) -> Self {
        return Self {
            burst32: data.into(),
            burst_sel: data.into(),
            lac: data.into(),
            popa: data.into(),
            bw: data.into(),
            sync_mode: data.into(),
            sync_pol: data.into(),
            dr_pol: data.into(),
        };
    }
}

impl Into<u16> for MscCtrl {
    fn into(self) -> u16 {
        let burst32: u16 = self.burst32.into();
        let burst_sel: u16 = self.burst_sel.into();
        let lac: u16 = self.lac.into();
        let popa: u16 = self.popa.into();
        let bw: u16 = self.bw.into();
        let sync_mode: u16 = self.sync_mode.into();
        let sync_pol: u16 = self.sync_pol.into();
        let dr_pol: u16 = self.dr_pol.into();

        return burst32 | burst_sel | lac | popa | bw | sync_mode | sync_pol | dr_pol;
    }
}

duplicate::duplicate! {
    [
        enum_name                         value0          value1        offset;
        [Burst32]                         [Disabled]      [Enabled]     [9];
        [BurstSel]                        [Sel0]          [Sel1]        [8];
        [LinearAccelerationCompensation]  [Disabled]      [Enabled]     [7];
        [PointOfPercussionAlignment]      [Disabled]      [Enabled]     [6];
        [SensorBandwidth]                 [Wide]          [Hz370]       [4];
        [SyncPolarity]                    [FallingEdge]   [RisingEdge]  [1];
        [DataReadyPolarity]               [ActiveLow]     [ActiveHigh]  [0];
    ]

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum enum_name {
        value0,
        value1,
    }

    impl From<u16> for enum_name {
        fn from(data: u16) -> Self {
            return match data & (1 << offset) != 0 {
                false => Self::value0,
                true => Self::value1,
            };
        }
    }

    impl Into<u16> for enum_name {
        fn into(self) -> u16 {
            return match self {
                Self::value0 => 0,
                Self::value1 => 1 << offset,
            };
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SyncMode {
    Internal,
    DirectInput,
    ScaledInput,
    Output,
}

impl From<u16> for SyncMode {
    fn from(data: u16) -> Self {
        return match (data >> 2) & 3 {
            0 => Self::Internal,
            1 => Self::DirectInput,
            2 => Self::ScaledInput,
            3 => Self::Output,
            _ => unreachable!(),
        };
    }
}

impl Into<u16> for SyncMode {
    fn into(self) -> u16 {
        return match self {
            Self::Internal => 0,
            Self::DirectInput => 1,
            Self::ScaledInput => 2,
            Self::Output => 3,
        } << 2;
    }
}

#[test]
fn msc_ctrl_test() {
    let msc_ctrl = MscCtrl::default();
    assert_eq!(msc_ctrl.burst32, Burst32::Disabled);
    assert_eq!(msc_ctrl.burst_sel, BurstSel::Sel0);
    assert_eq!(msc_ctrl.lac, LinearAccelerationCompensation::Enabled);
    assert_eq!(msc_ctrl.popa, PointOfPercussionAlignment::Enabled);
    assert_eq!(msc_ctrl.bw, SensorBandwidth::Wide);
    assert_eq!(msc_ctrl.sync_mode, SyncMode::Internal);
    assert_eq!(msc_ctrl.sync_pol, SyncPolarity::FallingEdge);
    assert_eq!(msc_ctrl.dr_pol, DataReadyPolarity::ActiveHigh);
}
