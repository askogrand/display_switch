# Build Instructions for Windows

## Prerequisites
Install Rust on Windows:
```bash
winget install Rustlang.Rust.MSVC
```

Or download from: https://rustup.rs/

## Build Steps
1. Copy this entire `source/` folder to your Windows machine
2. Open Command Prompt or PowerShell in the source folder
3. Run: `cargo build --release`
4. Binary will be created at: `target/release/display_switch.exe`

## Installation
1. Copy `display_switch.exe` to `C:\Program Files\DisplaySwitch\`
2. Copy `../windows/display-switch.ini` to same directory
3. Update USB device ID in config file
4. Create Windows service or startup task

## Windows Service (Optional)
Use `sc create` or Task Scheduler to run at startup:
```cmd
sc create DisplaySwitch binPath="C:\Program Files\DisplaySwitch\display_switch.exe" start=auto
```

## Notes
- Windows build uses standard DDC (slightly slower but reliable)
- Same LG detection and input source logic as macOS
- Lock screen switching should work once configured as service