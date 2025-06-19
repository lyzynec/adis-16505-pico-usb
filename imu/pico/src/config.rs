use super::adis::msc_ctrl::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    pub burst_enabled: bool,
    pub msc_ctrl: MscCtrl,
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            burst_enabled: false,
            msc_ctrl: MscCtrl::default(),
        };
    }
}
