//
// Linux-specific LG DDC implementation
//

use super::LgDdcHandle;
use anyhow::{anyhow, Result};

impl LgDdcHandle {
    pub(super) fn get_vcp_feature_lg_linux(&mut self, vcp_code: u8) -> Result<u16> {
        // TODO: Implement Linux DDC communication with source address 0x50 using i2c-dev
        log::debug!("Linux LG DDC read VCP {:#04x} - not yet implemented", vcp_code);
        Err(anyhow!("Linux LG DDC not yet implemented"))
    }

    pub(super) fn set_vcp_feature_lg_linux(&mut self, vcp_code: u8, value: u16) -> Result<()> {
        // TODO: Implement Linux DDC communication with source address 0x50 using i2c-dev
        log::debug!("Linux LG DDC write VCP {:#04x} = {} - not yet implemented", vcp_code, value);
        Err(anyhow!("Linux LG DDC not yet implemented"))
    }
}