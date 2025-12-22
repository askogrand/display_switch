# Display Switch with LG Support

Enhanced display switching tool with native LG display support featuring fast IOKit DDC communication on macOS.

## Features
- ✅ **LG vendor detection** - Automatic detection of LG displays
- ✅ **Fast IOKit DDC** on macOS for all LG inputs (DisplayPort + HDMI)  
- ✅ **Cross-platform support** - macOS, Windows, Linux
- ✅ **Lock screen switching** - Works when computer is locked
- ✅ **Background service** - Automatic startup and monitoring
- ✅ **Homebrew installation** - Secure package management

## Installation

### macOS (Recommended)

**Via Homebrew (Easiest):**
```bash
# Add the tap
brew tap askogrand/display-switch

# Install with LG support
brew install askogrand/display-switch/display_switch

# Configure (copy and edit the config file)
cp /opt/homebrew/etc/display-switch/display-switch.ini.example /opt/homebrew/etc/display-switch/display-switch.ini
# Edit config file with your USB device ID and monitor settings

# Start the service  
brew services start askogrand/display-switch/display_switch
```

**Manual Installation:**
```bash
sudo cp macos/display_switch /usr/local/bin/
cp macos/display-switch.ini ~/Library/Preferences/
cp macos/dev.haim.display-switch.daemon.plist ~/Library/LaunchAgents/
sudo launchctl load ~/Library/LaunchAgents/dev.haim.display-switch.daemon.plist
```

### Windows

**Build from source:**
```bash
# Install Rust: winget install Rustlang.Rust.MSVC
# Copy source/ folder to Windows machine
cd source/
cargo build --release
# Binary will be in: target/release/display_switch.exe
```

**Configuration:**
- Place `windows/display-switch.ini` in same directory as executable
- Update USB device IDs in config for your setup
- Create Windows service or startup task

## Configuration

**Example config for LG displays:**
```ini
# Your USB device ID (check with: display_switch --debug)
usb_device = "05e3:0626"

[monitor1]
monitor_id = "LG ULTRAGEAR+"
on_usb_connect = "LgHdmi2"         # Switch to HDMI2 when device connects
# on_usb_disconnect = "LgDisplayPort1"  # Optional: switch when disconnect
```

**Available LG Input Sources:**
- `LgHdmi1`, `LgHdmi2`, `LgHdmi3`, `LgHdmi4`
- `LgDisplayPort1`, `LgDisplayPort2`, `LgDisplayPort3`, `LgDisplayPort4`  
- `LgUsbC1`, `LgUsbC2`, `LgUsbC3`, `LgUsbC4`

## Troubleshooting

**Check if running:**
```bash
brew services list | grep display_switch
```

**View logs:**
```bash
tail -f /opt/homebrew/var/log/display-switch.log
```

**Test switching:**
```bash
/opt/homebrew/opt/display_switch/bin/display_switch --debug
```

## How It Works

1. **LG Detection** - Automatically detects LG displays by manufacturer ID and model name
2. **Enhanced DDC** - Uses LG-specific DDC protocol with source address 0x50 (not standard 0x51)
3. **Fast IOKit** - Direct IOKit communication on macOS for optimal performance
4. **Fallback Support** - Standard DDC fallback ensures compatibility
5. **USB Monitoring** - Watches for device connect/disconnect events

## Cross-Platform Deployment

- **macOS**: Fast IOKit DDC with LG-specific enhancements
- **Windows**: Standard DDC with LG protocol support (build from source)  
- **Linux**: Standard DDC with LG protocol support (source available)

