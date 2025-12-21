//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//

//! Low-level DDC communication with LG-specific support
//! This module implements direct DDC communication using platform-specific APIs
//! to support LG displays that require source address 0x50 instead of 0x51

use anyhow::Result;
use ddc_hi::{Ddc, Display, Handle};

/// Standard DDC source address
const DDC_SOURCE_ADDR_STANDARD: u8 = 0x51;

/// LG-specific DDC source address  
const DDC_SOURCE_ADDR_LG: u8 = 0x50;

/// Standard input select VCP code
const VCP_INPUT_SELECT_STANDARD: u8 = 0x60;

/// LG-specific input select VCP code
const VCP_INPUT_SELECT_LG: u8 = 0xF4;

/// Enhanced DDC handle that supports LG-specific communication
pub struct LgDdcHandle {
    pub standard_handle: Handle,
    pub display_id: String,
    pub is_lg: bool,
}

impl LgDdcHandle {
    pub fn new(display: Display, is_lg: bool) -> Self {
        let display_id = display.info.id.clone();
        let handle = display.handle;
        
        Self {
            standard_handle: handle,
            display_id,
            is_lg,
        }
    }

    /// Get the current input source, trying both standard and LG-specific VCP codes
    pub fn get_input_source(&mut self) -> Result<u16> {
        if self.is_lg {
            // For LG displays, first try the LG-specific VCP code
            if let Ok(result) = self.get_vcp_feature_lg(VCP_INPUT_SELECT_LG) {
                log::debug!("Got input source {} from LG VCP code", result);
                return Ok(result);
            }
            
            // Fallback to standard VCP code
            log::debug!("LG VCP code failed, trying standard VCP code");
        }
        
        // Use standard DDC for non-LG displays or LG fallback
        match self.standard_handle.get_vcp_feature(VCP_INPUT_SELECT_STANDARD) {
            Ok(feature) => Ok(feature.value()),
            Err(e) => Err(e.into()),
        }
    }

    /// Set input source, using appropriate method for LG vs standard displays
    pub fn set_input_source(&mut self, input_value: u16, is_lg_input: bool) -> Result<()> {
        if is_lg_input && self.is_lg {
            // Use LG-specific DDC communication
            log::debug!("Setting LG input {} using LG-specific DDC", input_value);
            self.set_vcp_feature_lg(VCP_INPUT_SELECT_LG, input_value)
        } else {
            // Use standard DDC communication
            log::debug!("Setting input {} using standard DDC", input_value);
            match self.standard_handle.set_vcp_feature(VCP_INPUT_SELECT_STANDARD, input_value) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Get VCP feature using LG-specific parameters (source address 0x50)
    fn get_vcp_feature_lg(&mut self, vcp_code: u8) -> Result<u16> {
        // Platform-specific implementation
        #[cfg(target_os = "macos")]
        {
            self.get_vcp_feature_lg_macos(vcp_code)
        }
        
        #[cfg(target_os = "windows")]
        {
            self.get_vcp_feature_lg_windows(vcp_code)
        }
        
        #[cfg(target_os = "linux")]
        {
            self.get_vcp_feature_lg_linux(vcp_code)
        }
    }

    /// Set VCP feature using LG-specific parameters (source address 0x50)
    fn set_vcp_feature_lg(&mut self, vcp_code: u8, value: u16) -> Result<()> {
        // Platform-specific implementation
        #[cfg(target_os = "macos")]
        {
            self.set_vcp_feature_lg_macos(vcp_code, value)
        }
        
        #[cfg(target_os = "windows")]
        {
            self.set_vcp_feature_lg_windows(vcp_code, value)
        }
        
        #[cfg(target_os = "linux")]
        {
            self.set_vcp_feature_lg_linux(vcp_code, value)
        }
    }
}

// Platform-specific implementations
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]  
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
use macos::*;

#[cfg(target_os = "windows")]
use windows::*;

#[cfg(target_os = "linux")]
use linux::*;