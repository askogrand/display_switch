# Display Switch Deployment

## LG Display Support
- ✅ LG vendor detection
- ✅ Fast IOKit DDC on macOS (DisplayPort + HDMI)
- ✅ Standard DDC fallback on Windows
- ✅ Lock screen switching support
- ✅ Background daemon operation

## macOS Deployment

### Files:
- `macos/display_switch` - Binary with fast IOKit DDC
- `macos/display-switch.ini` - Configuration file
- `macos/dev.haim.display-switch.daemon.plist` - Launch daemon

### Installation:
```bash
sudo cp macos/display_switch /usr/local/bin/
cp macos/display-switch.ini ~/Library/Preferences/
cp macos/dev.haim.display-switch.daemon.plist ~/Library/LaunchAgents/
sudo launchctl load ~/Library/LaunchAgents/dev.haim.display-switch.daemon.plist
```

## Windows Deployment

### Build from source:
```bash
# Install Rust: winget install Rustlang.Rust.MSVC
# Copy source/ folder to Windows machine
cd source/
cargo build --release
# Binary will be in: target/release/display_switch.exe
```

### Configuration:
- Place `windows/display-switch.ini` in same directory as executable
- Update USB device IDs in config for your setup
- Create Windows service or startup task

## Configuration Notes
- Update `usb_device = "27C6:639C"` with your actual USB device ID
- LG input mappings are pre-configured
- Both platforms use the same config format

## Testing
```bash
# Check USB devices
display_switch --debug

# Test switching (requires USB device connected)
sudo display_switch --debug
```