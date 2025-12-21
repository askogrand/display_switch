//
// macOS-specific LG DDC implementation using IOKit
// Implements proper low-level DDC with LG source address 0x50 support
//

use super::LgDdcHandle;
use anyhow::{anyhow, Result};
use core_graphics::display::CGDisplay;
use io_kit_sys::*;
use io_kit_sys::types::io_connect_t;
use std::ptr;
use std::os::raw::c_void;

// DDC/CI protocol constants
const DDC_ADDR: u8 = 0x37;
const DDC_REPLY_ADDR: u8 = 0x6E;
const DDC_SET_VCP_FEATURE: u8 = 0x02;
const DDC_GET_VCP_FEATURE: u8 = 0x01;
const DDC_GET_VCP_FEATURE_REPLY: u8 = 0x02;
const DDC_SOURCE_ADDR_LG: u8 = 0x50;

impl LgDdcHandle {
    pub(super) fn get_vcp_feature_lg_macos(&mut self, vcp_code: u8) -> Result<u16> {
        log::debug!("macOS LG DDC read VCP {:#04x} using IOKit with source address 0x50", vcp_code);
        
        let display_id = self.display.info.id.parse::<u32>()
            .map_err(|_| anyhow!("Failed to parse display ID: {}", self.display.info.id))?;
        
        let cg_display = CGDisplay::new(display_id);
        let service_port = unsafe { CGDisplayIOServicePort(cg_display.id) };
        
        if service_port == 0 {
            return Err(anyhow!("Failed to get display service port for display {}", display_id));
        }
        
        // Create DDC request packet with LG source address
        let mut request = vec![
            DDC_SOURCE_ADDR_LG,           // LG source address (0x50)
            6,                            // Message length
            DDC_GET_VCP_FEATURE,          // Get VCP feature command (0x01)
            vcp_code,                     // VCP code to read
        ];
        
        // Calculate DDC checksum: XOR of destination address + all data bytes
        let checksum = request.iter().fold(DDC_ADDR, |acc, &x| acc ^ x);
        request.push(checksum);
        
        log::debug!("Sending DDC GET request: {:02X?}", request);
        
        // Send DDC command via IOKit
        let mut output_count = 256u32;
        let mut output_buffer = [0u8; 256];
        
        let result = unsafe {
            IOConnectCallStructMethod(
                service_port as io_connect_t,
                0, // Method selector for DDC
                request.as_ptr() as *const c_void,
                request.len(),
                output_buffer.as_mut_ptr() as *mut c_void,
                &mut (output_count as usize) as *mut usize,
            )
        };
        
        if result != KERN_SUCCESS {
            return Err(anyhow!("DDC GET command failed with IOKit result: {:#08x}", result));
        }
        
        if output_count < 9 {
            return Err(anyhow!("DDC response too short: {} bytes", output_count));
        }
        
        let response = &output_buffer[0..output_count as usize];
        log::debug!("Received DDC GET response: {:02X?}", response);
        
        // Validate DDC response format: [reply_addr, length, cmd, result, max_hi, max_lo, cur_hi, cur_lo, checksum]
        if response[0] != DDC_REPLY_ADDR {
            return Err(anyhow!("Invalid DDC reply address: {:#02x}, expected {:#02x}", 
                             response[0], DDC_REPLY_ADDR));
        }
        
        if response[2] != DDC_GET_VCP_FEATURE_REPLY {
            return Err(anyhow!("Invalid DDC response command: {:#02x}, expected {:#02x}", 
                             response[2], DDC_GET_VCP_FEATURE_REPLY));
        }
        
        // Extract current value (bytes 6-7: high byte << 8 | low byte)
        let current_value = (response[6] as u16) << 8 | (response[7] as u16);
        
        log::debug!("VCP code {:#04X} current value: {:#04X} ({})", vcp_code, current_value, current_value);
        Ok(current_value)
    }

    pub(super) fn set_vcp_feature_lg_macos(&mut self, vcp_code: u8, value: u16) -> Result<()> {
        log::debug!("macOS LG DDC write VCP {:#04x} = {:#04x} using IOKit with source address 0x50", vcp_code, value);
        
        let display_id = self.display.info.id.parse::<u32>()
            .map_err(|_| anyhow!("Failed to parse display ID: {}", self.display.info.id))?;
        
        let cg_display = CGDisplay::new(display_id);
        let service_port = unsafe { CGDisplayIOServicePort(cg_display.id) };
        
        if service_port == 0 {
            return Err(anyhow!("Failed to get display service port for display {}", display_id));
        }
        
        // Create DDC SET request packet with LG source address
        let value_high = (value >> 8) as u8;
        let value_low = (value & 0xFF) as u8;
        
        let mut request = vec![
            DDC_SOURCE_ADDR_LG,           // LG source address (0x50)
            8,                            // Message length
            DDC_SET_VCP_FEATURE,          // Set VCP feature command (0x02)
            vcp_code,                     // VCP code to set
            value_high,                   // Value high byte
            value_low,                    // Value low byte
        ];
        
        // Calculate DDC checksum
        let checksum = request.iter().fold(DDC_ADDR, |acc, &x| acc ^ x);
        request.push(checksum);
        
        log::debug!("Sending DDC SET request: {:02X?} for VCP {:#04X} = {:#04X}", 
                   request, vcp_code, value);
        
        // Send DDC command via IOKit
        let result = unsafe {
            IOConnectCallStructMethod(
                service_port as io_connect_t,
                0, // Method selector for DDC
                request.as_ptr() as *const c_void,
                request.len(),
                ptr::null_mut(),
                ptr::null_mut::<usize>(),
            )
        };
        
        if result != KERN_SUCCESS {
            return Err(anyhow!("DDC SET command failed with IOKit result: {:#08x}", result));
        }
        
        log::debug!("Successfully sent DDC SET command for VCP {:#04X} = {:#04X}", vcp_code, value);
        
        // Give the display time to process the command
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }
}