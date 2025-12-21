//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//
use crate::configuration::{Configuration, SwitchDirection};
use crate::input_source::InputSource;
use crate::lg_ddc::LgDdcHandle;
use crate::vendor;

use anyhow::{Error, Result};
use ddc_hi::{Ddc, Display, Handle};

use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::{thread, time};

/// VCP feature code for input select (standard)
const INPUT_SELECT: u8 = 0x60;
/// VCP feature code for LG-specific input select
const LG_INPUT_SELECT: u8 = 0xF4;
const RETRY_DELAY_MS: u64 = 3000;

fn display_name(display: &Display, index: Option<usize>) -> String {
    // Different OSes populate different fields of ddc-hi-rs info structure differently. Create
    // a synthetic "display_name" that makes sense on each OS
    #[cfg(target_os = "linux")]
    let display_id = vec![
        &display.info.manufacturer_id,
        &display.info.model_name,
        &display.info.serial_number,
    ]
    .into_iter()
    .flatten()
    .map(|s| s.as_str())
    .collect::<Vec<&str>>()
    .join(" ");
    #[cfg(target_os = "macos")]
    let display_id = &display.info.id;
    #[cfg(target_os = "windows")]
    let display_id = &display.info.id;

    if let Some(index) = index {
        format!("'{} #{}'", display_id, index)
    } else {
        format!("'{}'", display_id)
    }
}

fn are_display_names_unique(displays: &[Display]) -> bool {
    let mut hash = HashSet::new();
    displays.iter().all(|display| hash.insert(display_name(display, None)))
}

fn try_switch_display(handle: &mut Handle, is_lg: bool, display_name_str: &str, input: InputSource) {
    // If user configured LG-specific inputs, assume they know it's an LG display
    let treat_as_lg = is_lg || input.is_lg_specific();
    
    if treat_as_lg && input.is_lg_specific() {
        // For LG-specific inputs, use our low-level LG DDC implementation
        debug!("Attempting to switch LG display {} to {} using native LG DDC", display_name_str, input);
        
        if let Some(display) = ddc_hi::Display::enumerate().into_iter()
            .find(|d| {
                let d_name = display_name(&d, None);
                let target_name = &display_name_str[1..display_name_str.len()-1];
                d_name.contains(target_name)
            }) 
        {
            let mut lg_handle = LgDdcHandle::new(display, true);
            
            // Check if already on target input
            match lg_handle.get_input_source() {
                Ok(current) if current == input.value() => {
                    info!("LG display {} is already set to {}", display_name_str, input);
                    return;
                }
                Ok(current) => {
                    debug!("LG display {} currently on input {:#04X}, switching to {:#04X}", 
                           display_name_str, current, input.value());
                }
                Err(e) => {
                    warn!("Failed to get current LG input: {}", e);
                }
            }
            
            // Set the new input using LG DDC protocol
            match lg_handle.set_input_source(input.value(), true) {
                Ok(_) => {
                    info!("LG display {} set to {} using native LG DDC", display_name_str, input);
                    return;
                }
                Err(e) => {
                    warn!("Native LG DDC failed: {}. Falling back to standard DDC", e);
                }
            }
        } else {
            warn!("Could not find display for LG DDC, falling back to standard DDC");
        }
    }
    
    // Use standard DDC approach for non-LG displays or as fallback
    let vcp_code = if treat_as_lg && input.is_lg_specific() { 
        LG_INPUT_SELECT 
    } else { 
        INPUT_SELECT 
    };
    
    // Check current input
    match handle.get_vcp_feature(vcp_code) {
        Ok(raw_source) => {
            if raw_source.value() & 0xff == input.value() {
                info!("Display {} is already set to {}", display_name_str, input);
                return;
            }
        }
        Err(err) => {
            warn!("Failed to get current input for display {} using VCP {:#04X}: {:?}", 
                  display_name_str, vcp_code, err);
        }
    }
    
    debug!("Setting display {} to {} using VCP {:#04X}", display_name_str, input, vcp_code);
    match handle.set_vcp_feature(vcp_code, input.value()) {
        Ok(_) => {
            info!("Display {} set to {}", display_name_str, input);
        }
        Err(err) => {
            error!("Failed to set display {} to {} using VCP {:#04X} ({:?})", 
                   display_name_str, input, vcp_code, err);
                   
            // Final fallback: try standard VCP for LG displays
            if treat_as_lg && vcp_code == LG_INPUT_SELECT {
                warn!("Falling back to standard VCP code for LG display {}", display_name_str);
                match handle.set_vcp_feature(INPUT_SELECT, input.value()) {
                    Ok(_) => {
                        info!("Display {} set to {} using standard VCP fallback", display_name_str, input);
                    }
                    Err(fallback_err) => {
                        error!("All methods failed for display {}: {:?}", display_name_str, fallback_err);
                    }
                }
            }
        }
    }
}

fn displays() -> Vec<Display> {
    let displays = Display::enumerate();
    if !displays.is_empty() {
        return displays;
    }

    // Under some conditions, such as when using a KVM, it's possible for the USB connection/disconnection events to
    // occur before the display(s) become available. We retry once after a bit of a delay in order to be more
    // forgiving with regard to timing.
    let delay_duration = time::Duration::from_millis(RETRY_DELAY_MS);
    warn!(
        "Did not detect any DDC-compatible displays. Retrying after {} second(s)...",
        delay_duration.as_secs()
    );
    thread::sleep(delay_duration);
    Display::enumerate()
}

pub fn log_current_source() {
    let displays = displays();
    if displays.is_empty() {
        error!("Did not detect any DDC-compatible displays!");
        return;
    }
    let unique_names = are_display_names_unique(&displays);
    for (index, mut display) in displays.into_iter().enumerate() {
        let display_name = display_name(&display, if unique_names { None } else { Some(index + 1) });
        let is_lg = vendor::is_lg_display(&display);
        
        // Try both standard and LG-specific VCP codes for LG displays
        let mut found_input = false;
        
        // First try the appropriate VCP code
        let primary_vcp = if is_lg { LG_INPUT_SELECT } else { INPUT_SELECT };
        match display.handle.get_vcp_feature(primary_vcp) {
            Ok(raw_source) => {
                let source = InputSource::from(raw_source.value());
                info!("Display {} is currently set to {} (VCP {:#04X})", 
                      display_name, source, primary_vcp);
                found_input = true;
            }
            Err(err) => {
                debug!("Failed to get current input for display {} using VCP {:#04X}: {:?}", 
                       display_name, primary_vcp, err);
            }
        }
        
        // For LG displays, also try the standard VCP if the LG-specific one failed
        if is_lg && !found_input {
            match display.handle.get_vcp_feature(INPUT_SELECT) {
                Ok(raw_source) => {
                    let source = InputSource::from(raw_source.value());
                    info!("Display {} is currently set to {} (VCP {:#04X} fallback)", 
                          display_name, source, INPUT_SELECT);
                    found_input = true;
                }
                Err(err) => {
                    debug!("Failed to get current input for display {} using fallback VCP {:#04X}: {:?}", 
                           display_name, INPUT_SELECT, err);
                }
            }
        }
        
        if !found_input {
            error!("Failed to get current input for display {} using any VCP code", display_name);
        }
    }
}

pub fn switch(config: &Configuration, switch_direction: SwitchDirection) {
    let displays = displays();
    if displays.is_empty() {
        error!("Did not detect any DDC-compatible displays!");
        return;
    }
    let unique_names = are_display_names_unique(&displays);
    for (index, mut display) in displays.into_iter().enumerate() {
        let display_name = display_name(&display, if unique_names { None } else { Some(index + 1) });
        let is_lg = vendor::is_lg_display(&display); // Extract vendor info before mutable borrow
        let input_sources = config.configuration_for_monitor(&display_name);
        debug!("Input sources found for display {}: {:?}", display_name, input_sources);
        if let Some(input) = input_sources.source(switch_direction) {
            try_switch_display(&mut display.handle, is_lg, &display_name, input);
        } else {
            info!(
                "Display {} is not configured to switch on USB {}",
                display_name, switch_direction
            );
        }
        if let Some(execute_command) = input_sources.execute_command(switch_direction) {
            run_command(execute_command)
        }
    }
    if let Some(execute_command) = config.default_input_sources.execute_command(switch_direction) {
        run_command(execute_command)
    }
}

fn run_command(execute_command: &str) {
    fn try_run_command(execute_command: &str) -> Result<()> {
        let mut arguments = shell_words::split(execute_command)?;
        if arguments.is_empty() {
            return Ok(());
        }

        let executable = arguments.remove(0);
        let output = Command::new(executable).args(arguments).stdin(Stdio::null()).output()?;
        let stdout = if !output.stdout.is_empty() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                format!("Stdout = [{}]\n", s)
            } else {
                "Stdout was not UTF-8".to_string()
            }
        } else {
            "No stdout\n".to_string()
        };
        let stderr = if !output.stderr.is_empty() {
            if let Ok(s) = String::from_utf8(output.stderr) {
                format!("Stderr = [{}]\n", s)
            } else {
                "Stderr was not UTF-8".to_string()
            }
        } else {
            "No stderr\n".to_string()
        };

        if output.status.success() {
            debug!("External command output: {} {}", stdout, stderr);
            info!("External command '{}' executed successfully", execute_command);
            Ok(())
        } else {
            let msg = if let Some(code) = output.status.code() {
                format!("Exited with status {}\n", code)
            } else {
                "Exited because of a signal\n".to_string()
            };
            Err(Error::msg(format!("{} {} {}", msg, stdout, stderr)))
        }
    }

    try_run_command(execute_command)
        .unwrap_or_else(|err| error!("Error executing external command '{}': {}", execute_command, err))
}
