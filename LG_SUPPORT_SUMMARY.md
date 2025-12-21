# LG Display Support Implementation Summary

This document summarizes the changes made to add LG-specific display support to display-switch.

## Problem
LG displays often don't respond to standard DDC/CI input switching commands using the standard VCP code 0x60. This is a known issue with many LG monitor models where standard DDC input switching only blinks the monitor but doesn't actually change the input.

## Solution
Based on the implementation in the Lunar project, we've added LG-specific DDC support that:

1. **Detects LG displays** using manufacturer information from EDID
2. **Uses LG-specific DDC commands** (VCP code 0xF4 instead of 0x60)
3. **Supports LG-specific input source values** with different hex codes
4. **Provides fallback behavior** to standard DDC if LG-specific commands fail

## Changes Made

### 1. Vendor Detection (`src/vendor.rs`)
- New module to detect display vendors
- `detect_vendor()` function that identifies LG displays based on manufacturer ID
- `is_lg_display()` helper function
- Uses manufacturer ID patterns like "GSM" or "LG" to identify LG displays

### 2. Enhanced Input Sources (`src/input_source.rs`)
- Added LG-specific input source variants:
  - `LgDisplayPort1-4` (0xD0, 0xD1, 0xC0, 0xC1)
  - `LgHdmi1-4` (0x90, 0x91, 0x92, 0x93) 
  - `LgUsbC1-4` (0xD2, 0xD3, 0xE0, 0xE1)
- Added `is_lg_specific()` method to check if input source is LG-specific
- Maintains backward compatibility with existing input sources

### 3. Enhanced Display Control (`src/display_control.rs`)
- Added `LG_INPUT_SELECT` constant (0xF4) for LG-specific VCP code
- Modified `try_switch_display()` to:
  - Detect LG displays using vendor detection
  - Use appropriate VCP code (0xF4 for LG-specific inputs, 0x60 for standard)
  - Provide fallback to standard VCP if LG-specific command fails
  - Validate input source compatibility with display type
- Enhanced `log_current_source()` to try both VCP codes on LG displays

### 4. Documentation Updates
- Updated README.md with LG-specific input source documentation
- Added explanation of the LG display issue and solution
- Provided configuration examples for different LG models
- Created example configuration file (`display-switch-lg-example.ini`)

## Usage

### For LG Displays
```ini
usb_device = "1050:0407" 
on_usb_connect = "LgDisplayPort1"
on_usb_disconnect = "LgHdmi1"
```

### Mixed Environment (LG + Standard Displays)
```ini
usb_device = "1050:0407"
on_usb_connect = "DisplayPort1"  # Default for non-LG

[lg-monitor]
monitor_id = "LG"
on_usb_connect = "LgDisplayPort1"  # LG-specific

[standard-monitor]
monitor_id = "DELL" 
on_usb_connect = "DisplayPort2"  # Standard DDC
```

## Technical Details

### LG DDC Protocol
- Uses VCP code 0xF4 (MANUFACTURER_SPECIFIC_F4) instead of standard 0x60
- Based on Lunar's implementation which addresses LG's proprietary DDC behavior
- Includes fallback to standard DDC for compatibility

### Input Source Values
LG uses different hex values for input sources compared to VESA standard:
- Standard HDMI1: 0x11 → LG HDMI1: 0x90  
- Standard DisplayPort1: 0x0F → LG DisplayPort1: 0xD0
- And so on...

### Vendor Detection
- Currently uses manufacturer ID string matching ("GSM", "LG")
- Could be enhanced with EDID vendor ID (7789 for LG) if access to raw EDID is available

## Compatibility
- Fully backward compatible with existing configurations
- Standard displays continue to work with standard input sources
- LG displays can use either LG-specific or standard input sources
- Automatic detection means no user configuration required for vendor-specific behavior

## Testing
To test the implementation:
1. Use `display-switch --debug` to see vendor detection and VCP code selection
2. Try both LG-specific and standard input sources on LG displays
3. Verify fallback behavior works when LG-specific commands fail
4. Test mixed environments with both LG and non-LG displays