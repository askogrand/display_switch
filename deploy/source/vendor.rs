//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use ddc_hi::Display;

/// LG vendor ID based on EDID specification
const LG_VENDOR_ID: u16 = 7789;

/// Vendor enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vendor {
    LG,
    Unknown,
}

/// Detect vendor from display information
pub fn detect_vendor(display: &Display) -> Vendor {
    // Debug: print what info we have about the display
    log::debug!("Vendor detection for display: manufacturer_id={:?}, model_name={:?}, id={:?}", 
                display.info.manufacturer_id, display.info.model_name, display.info.id);
    
    // Try to extract vendor ID from manufacturer_id string if available
    if let Some(manufacturer_id) = &display.info.manufacturer_id {
        // Manufacturer ID in EDID is typically a 3-character string
        // LG uses "GSM" as their manufacturer ID in some cases
        if manufacturer_id.to_uppercase().contains("GSM") || 
           manufacturer_id.to_uppercase().contains("LG") {
            log::debug!("LG detected via manufacturer_id: {}", manufacturer_id);
            return Vendor::LG;
        }
    }

    // Try to parse vendor ID from model name or other fields if needed
    if let Some(model_name) = &display.info.model_name {
        if model_name.to_uppercase().contains("LG") {
            log::debug!("LG detected via model_name: {}", model_name);
            return Vendor::LG;
        }
    }

    // Check the display ID/name patterns that indicate LG displays
    // macOS often shows LG displays with specific patterns
    #[cfg(target_os = "macos")]
    {
        let display_id = &display.info.id;
        if display_id.to_uppercase().contains("LG") ||
           display_id.to_uppercase().contains("ULTRAGEAR") ||
           display_id.to_uppercase().contains("ULTRAWIDE") {
            log::debug!("LG detected via display ID: {}", display_id);
            return Vendor::LG;
        }
    }

    // For more robust detection, we would need to access the raw EDID data
    // which might not be available through ddc-hi's API. This is a fallback
    // approach using available string fields.
    
    log::debug!("No LG pattern detected, marking as Unknown vendor");
    Vendor::Unknown
}

/// Check if a display is manufactured by LG
pub fn is_lg_display(display: &Display) -> bool {
    detect_vendor(display) == Vendor::LG
}