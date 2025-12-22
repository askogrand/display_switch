//
// macOS-specific LG DDC implementation using IOKit
// Full DDC communication with proper LG source address 0x50 support
//

use super::LgDdcHandle;
use anyhow::{anyhow, Result};
use ddc_hi::Ddc;

extern crate mach2;
extern crate io_kit_sys;
extern crate core_foundation;

use std::ffi::{c_void, CString};
use std::ptr;
use mach2::mach_types::*;
use mach2::kern_return::*;
use mach2::traps::mach_task_self;
use io_kit_sys::*;
use io_kit_sys::types::*;
use core_foundation::base::*;
use core_foundation::string::*;
use core_foundation::dictionary::*;

// DDC Constants for LG-specific communication
const DDC_DEST_ADDR: u8 = 0x37;         // Standard destination address  
const DDC_SOURCE_ADDR_LG: u8 = 0x50;    // LG-specific source address (not 0x51!)
const DDC_VCP_GET: u8 = 0x01;
const DDC_VCP_SET: u8 = 0x02;
const LG_INPUT_SOURCE_VCP: u8 = 0xF4;    // LG uses 0xF4 instead of 0x60

// DDC packet structure with LG-specific source address
#[repr(C)]
struct DdcWriteCommand {
    send_addr: u8,
    send_size: u32,
    send_buffer: [u8; 128],
}

impl LgDdcHandle {
    pub(super) fn get_vcp_feature_lg_macos(&mut self, vcp_code: u8) -> Result<u16> {
        log::debug!("macOS LG DDC read VCP {:#04x} with source address 0x50", vcp_code);
        
        // Try IOKit DDC first with proper LG source address 0x50
        match self.iokit_ddc_read(vcp_code) {
            Ok(value) => {
                log::debug!("IOKit LG DDC read successful: VCP {:#04x} = {:#04x}", vcp_code, value);
                return Ok(value);
            }
            Err(e) => {
                log::warn!("IOKit LG DDC read failed: {}, falling back to standard DDC", e);
            }
        }
        
        // Fall back to standard DDC
        match self.standard_handle.get_vcp_feature(vcp_code) {
            Ok(feature) => {
                log::debug!("Standard DDC fallback returned {:#04x} for VCP {:#04x}", feature.value(), vcp_code);
                Ok(feature.value())
            }
            Err(e) => {
                log::warn!("Standard DDC fallback also failed for VCP {:#04x}: {}", vcp_code, e);
                Err(anyhow!("LG DDC read failed: {}", e))
            }
        }
    }

    pub(super) fn set_vcp_feature_lg_macos(&mut self, vcp_code: u8, value: u16) -> Result<()> {
        log::debug!("macOS LG DDC write VCP {:#04x} = {:#04x} with source address 0x50", vcp_code, value);
        
        // Try IOKit DDC first with proper LG source address 0x50
        match self.iokit_ddc_write(vcp_code, value) {
            Ok(_) => {
                log::info!("IOKit LG DDC write successful: VCP {:#04x} = {:#04x}", vcp_code, value);
                // Add delay for display processing
                std::thread::sleep(std::time::Duration::from_millis(250));
                return Ok(());
            }
            Err(e) => {
                log::warn!("IOKit LG DDC write failed: {}, falling back", e);
            }
        }
        
        // Try standard DDC (might work for HDMI inputs)
        match self.standard_handle.set_vcp_feature(vcp_code, value) {
            Ok(_) => {
                log::debug!("Standard DDC fallback succeeded for VCP {:#04x} = {:#04x}", vcp_code, value);
                std::thread::sleep(std::time::Duration::from_millis(250));
                Ok(())
            }
            Err(e) => {
                log::warn!("Standard DDC fallback failed for VCP {:#04x} = {:#04x}: {}", vcp_code, value, e);
                
                // For DisplayPort switching, use lunar CLI as last resort
                if vcp_code == 0xF4 && Self::is_displayport_value(value) {
                    log::warn!("DisplayPort switching failed with DDC, using lunar CLI workaround");
                    self.call_lunar_for_displayport(value)
                } else {
                    Err(anyhow!("All DDC methods failed: {}", e))
                }
            }
        }
    }
    
    // IOKit DDC read with LG source address 0x50
    fn iokit_ddc_read(&self, vcp_code: u8) -> Result<u16> {
        // Create DDC read packet with LG source address
        let mut packet = vec![
            DDC_DEST_ADDR | 0x80,  // destination with read bit
            DDC_SOURCE_ADDR_LG,     // LG source address 0x50  
            DDC_VCP_GET,            // GET command
            vcp_code,               // VCP code to read
        ];
        
        // Calculate length and checksum
        let length = packet.len() as u8;
        packet.insert(0, length);
        
        let mut checksum: u8 = 0x50;
        for &byte in &packet {
            checksum ^= byte;
        }
        packet.push(checksum);
        
        log::debug!("LG DDC read packet: {:02X?}", packet);
        
        // Get display service and send command
        match self.send_iokit_ddc_command(&packet, true) {
            Ok(Some(response)) => {
                if response.len() >= 10 {
                    // Parse DDC response: [length] [dest] [source] [cmd] [result] [type] [max_hi] [max_lo] [cur_hi] [cur_lo] [checksum]
                    let current_value = ((response[8] as u16) << 8) | (response[9] as u16);
                    log::debug!("LG DDC read response: current = {:#04x}", current_value);
                    Ok(current_value)
                } else {
                    Err(anyhow!("Invalid DDC read response length: {}", response.len()))
                }
            }
            Ok(None) => Err(anyhow!("No response from LG DDC read command")),
            Err(e) => Err(anyhow!("IOKit DDC read failed: {}", e)),
        }
    }
    
    // IOKit DDC write with LG source address 0x50
    fn iokit_ddc_write(&self, vcp_code: u8, value: u16) -> Result<()> {
        // Create DDC write packet with LG source address
        let mut packet = vec![
            DDC_DEST_ADDR | 0x80,   // destination with write bit
            DDC_SOURCE_ADDR_LG,     // LG source address 0x50
            DDC_VCP_SET,            // SET command  
            vcp_code,               // VCP code
            (value >> 8) as u8,     // value high byte
            value as u8,            // value low byte
        ];
        
        // Calculate length and checksum
        let length = packet.len() as u8;
        packet.insert(0, length);
        
        let mut checksum: u8 = 0x50;
        for &byte in &packet {
            checksum ^= byte;
        }
        packet.push(checksum);
        
        log::debug!("LG DDC write packet: {:02X?}", packet);
        
        // Get display service and send command
        match self.send_iokit_ddc_command(&packet, false) {
            Ok(_) => {
                log::debug!("LG DDC write command sent successfully");
                Ok(())
            }
            Err(e) => Err(anyhow!("IOKit DDC write failed: {}", e)),
        }
    }
    
    // Send DDC command via IOKit
    fn send_iokit_ddc_command(&self, packet: &[u8], expect_response: bool) -> Result<Option<Vec<u8>>> {
        unsafe {
            // Get display service
            let service = self.get_display_service()?;
            
            // Open connection
            let mut connection: io_connect_t = 0;
            let kr = IOServiceOpen(service, mach_task_self(), 0, &mut connection);
            IOObjectRelease(service);
            
            if kr != KERN_SUCCESS {
                return Err(anyhow!("Failed to open IOService connection: {}", kr));
            }
            
            // Prepare DDC command structure
            let mut ddc_command = DdcWriteCommand {
                send_addr: DDC_DEST_ADDR,
                send_size: packet.len() as u32,
                send_buffer: [0; 128],
            };
            
            // Copy packet to send buffer
            ddc_command.send_buffer[..packet.len()].copy_from_slice(packet);
            
            let mut output_buffer = vec![0u8; 256];
            let mut output_size = output_buffer.len();
            
            // Send DDC command via IOKit
            let kr = IOConnectCallMethod(
                connection,
                0, // DDC command selector
                ptr::null(),
                0,
                &ddc_command as *const _ as *const c_void,
                std::mem::size_of::<DdcWriteCommand>(),
                ptr::null_mut(),
                ptr::null_mut(),
                if expect_response { output_buffer.as_mut_ptr() as *mut c_void } else { ptr::null_mut() },
                if expect_response { &mut output_size } else { ptr::null_mut() },
            );
            
            IOServiceClose(connection);
            
            if kr != KERN_SUCCESS {
                return Err(anyhow!("IOKit DDC call failed with error: {}", kr));
            }
            
            if expect_response && output_size > 0 {
                output_buffer.truncate(output_size);
                Ok(Some(output_buffer))
            } else {
                Ok(None)
            }
        }
    }
    
    // Get display IOService for DDC communication  
    fn get_display_service(&self) -> Result<io_service_t> {
        unsafe {
            let service_name = CString::new("IOFramebufferI2CInterface")?;
            let matching_dict = IOServiceMatching(service_name.as_ptr());
            
            if matching_dict.is_null() {
                return Err(anyhow!("Failed to create IOService matching dictionary"));
            }
            
            let mut iterator: io_iterator_t = 0;
            let kr = IOServiceGetMatchingServices(kIOMasterPortDefault, matching_dict, &mut iterator);
            
            if kr != KERN_SUCCESS {
                return Err(anyhow!("Failed to get matching IOServices: {}", kr));
            }
            
            let service = IOIteratorNext(iterator);
            IOObjectRelease(iterator);
            
            if service == 0 {
                return Err(anyhow!("No display IOServices found"));
            }
            
            Ok(service)
        }
    }
    
    fn is_displayport_value(value: u16) -> bool {
        matches!(value, 0xD0 | 0xD1 | 0xC0 | 0xC1)
    }
    
    fn call_lunar_for_displayport(&self, value: u16) -> Result<()> {
        use std::process::{Command, Stdio};
        
        let lunar_input = match value {
            0xD0 => "lgDisplayPort1",
            0xD1 => "lgDisplayPort2", 
            0xC0 => "lgDisplayPort3",
            0xC1 => "lgDisplayPort4",
            _ => return Err(anyhow!("Unknown DisplayPort value: {:#04x}", value)),
        };
        
        let lunar_path = std::env::var("HOME")
            .map(|home| format!("{}/.local/bin/lunar", home))
            .unwrap_or_else(|_| "/usr/local/bin/lunar".to_string());
            
        log::debug!("Calling lunar: {} displays lg input {}", lunar_path, lunar_input);
        
        match Command::new(&lunar_path)
            .args(&["displays", "lg", "input", lunar_input])
            .stdin(Stdio::null())
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::debug!("Lunar DisplayPort command succeeded");
                    Ok(())
                } else {
                    log::warn!("Lunar DisplayPort command failed with status {}: {}", 
                             output.status, String::from_utf8_lossy(&output.stderr));
                    Err(anyhow!("Lunar DisplayPort command failed"))
                }
            }
            Err(err) => {
                log::warn!("Failed to execute lunar DisplayPort command: {}", err);
                Err(anyhow!("Failed to execute lunar: {}", err))
            }
        }
    }
}

// External IOKit function declarations
extern "C" {
    fn IOServiceMatching(name: *const std::os::raw::c_char) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        master_port: mach_port_t,
        matching: CFDictionaryRef,
        existing: *mut io_iterator_t,
    ) -> kern_return_t;
    fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;
    fn IOObjectRelease(object: io_object_t) -> kern_return_t;
    fn IOServiceOpen(
        service: io_service_t, 
        owning_task: task_port_t, 
        type_: u32, 
        connect: *mut io_connect_t
    ) -> kern_return_t;
    fn IOServiceClose(connect: io_connect_t) -> kern_return_t;
    fn IOConnectCallMethod(
        connection: io_connect_t,
        selector: u32,
        input: *const u64,
        input_cnt: u32,
        input_struct: *const c_void,
        input_struct_size: usize,
        output: *mut u64,
        output_cnt: *mut u32,
        output_struct: *mut c_void,
        output_struct_size: *mut usize,
    ) -> kern_return_t;
}