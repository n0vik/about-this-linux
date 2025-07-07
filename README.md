# About This Linux

A highly customizable "About this Mac" dialog for Linux systems, now rewritten in Rust with GTK4.

**Version:** 0.9.0
**Author:** Kamil 'Novik' Nowicki <kamil.nowicki@h4b.uk>  
**License:** GPL-3.0-or-later

## Introduction

This is a complete rewrite of the original AboutThisMc project in Rust using GTK4. It provides a macOS-style "About this Mac" window for Linux systems with the same visual appearance and functionality, but with improved performance, memory safety, and modern GTK4 features.

### Key Improvements in v0.3.0
- **Complete rewrite in Rust** for better performance and memory safety
- **GTK4 support** with modern UI components
- **Graphical configuration wizard** - no more terminal-based setup!
- **Improved system detection** with better error handling
- **Cleaner codebase** with proper separation of concerns
- **Better versioning** and author information display

## Features

- Mimics the macOS "About this Mac" dialog appearance
- Automatic system information detection using `fastfetch` and `dmidecode`
- Graphical configuration wizard with the same look as the main application
- Customizable distro logos and system information
- Multiple configuration profiles support
- Tabbed interface (Overview, Display, Storage, Support, Service)
- Modern GTK4 interface with smooth animations

## Dependencies

- **System tools:**
  - `fastfetch` - for system information detection
  - `dmidecode` - for hardware information (requires sudo access)
  - `lsblk` - for disk information

- **Development dependencies:**
  - Rust 1.70+ with Cargo
  - GTK4 development libraries
  - GDK-Pixbuf development libraries

### Installing Dependencies on Arch Linux

```bash
# System tools
sudo pacman -S fastfetch dmidecode util-linux

# Development dependencies (if building from source)
sudo pacman -S rust gtk4 gdk-pixbuf2
```

### Installing Dependencies on Ubuntu/Debian

```bash
# System tools
sudo apt install fastfetch dmidecode util-linux

# Development dependencies (if building from source)
sudo apt install rustc cargo libgtk-4-dev libgdk-pixbuf-2.0-dev
```

## Installation

### From Source

1. Clone this repository:
```bash
git clone <repository-url>
cd about-this-linux-v2
```

2. Build the application:
```bash
cargo build --release
```

3. Install the binary:
```bash
sudo cp target/release/about-this-linux /usr/local/bin/
```

### Running the Application

```bash
# Run normally (will show configurator on first run)
about-this-linux

# Force configuration wizard
about-this-linux --configure

# Load a specific configuration file
about-this-linux --load-overview /path/to/config.json

# Use a custom config path
about-this-linux --config-path /path/to/custom/config.json
```

## Configuration

### First-Time Setup

On the first run, the application will launch a graphical configuration wizard with the same appearance as the main window. The wizard includes:

1. **Auto Detection Tab** - Automatically detects system information
2. **Manual Config Tab** - Allows manual entry of all system details
3. **Preview Tab** - Preview how your configuration will look
4. **About Tab** - Information about the application

### Configuration File

The application stores configuration in JSON format at:
`~/.local/share/about-this-linux/overview-conf.json`

#### Configuration Options

- `distro_image_path`: Path to the distro logo image
- `distro_image_size`: Array of [width, height] for the logo
- `distro_markup`: Pango markup for the distro name display
- `distro_ver`: Distribution version string
- `hostname`: Device name/model
- `cpu`: Processor information
- `memory`: RAM information
- `startup_disk`: Boot disk information
- `graphics`: GPU information
- `serial_num`: System serial number
- `overview_margins`: Array of [left, right, top, bottom] margins
- `section_space`: Spacing between sections
- `logo_space`: Space between logo and information
- `system_info_command`: Command for "System Report" button
- `software_update_command`: Command for "Software Update" button
- `font-family`: Font family (optional)

- Sample config file 1:
    ```json
    {
    "distro_image_path": "/home/ngocphat/local/share/about-this-mc/distro-logo.png",
    "distro_image_size": [
        160,
        160
    ],
    "distro_markup": "<span font-size='xx-large'><span font-weight='bold'>Arch Linux</span></span>",
    "distro_ver": "5.10.11-arch1-1",
    "hostname": "20UD0001CD ThinkPad T14 Gen 1",
    "cpu": "2.100GHz AMD Ryzen 5 PRO 4650U",
    "memory": "16.0 GB 3200 MHz DDR4",
    "startup_disk": "nvme0n1p1",
    "graphics": "AMD ATI 07:00.0 Renoir",
    "serial_num": "L1XXXXX02GE",
    "overview_margins": [
        60,
        60,
        60,
        60
    ],
    "section_space": 20,
    "logo_space": 60,
    "system_info_command": "",
    "software_update_command": "",
    "font-family": null
    }
    ```

- Sample config file 2 (for faking a MacBook on macOS rices):
    ```json
    {
    "distro_image_path": "/home/ngocphat/local/share/about-this-mc/bigsur.png",
    "distro_image_size": [
        160,
        160
    ],
    "distro_markup": "<span font-size='xx-large'><span font-weight='bold'>macOS </span>Big Sur</span>",
    "distro_ver": "Version 11.0.1",
    "hostname": "MacBook Pro 2020 (Early, 13-inch)",
    "cpu": "2.100GHz AMD Ryzen 5 PRO 4650U",
    "memory": "16.0 GB 3200 MHz DDR4",
    "startup_disk": "Macintosh HD",
    "graphics": "AMD Radeon RX Vega 5 2 GB",
    "serial_num": "XXXXXXXXXXXXXX",
    "overview_margins": [
        60,
        60,
        60,
        60
    ],
    "section_space": 20,
    "logo_space": 60,
    "system_info_command": "",
    "software_update_command": "",
    "font-family": "San Francisco Display"
    }
    ```

# Roadmap
- ~~Implement font changing.~~
- ~~Implement "Storage" tab.~~

# Changelog
```
v0.9.0 
- Removed dependency on configuration files; automatic system information detection.
- System-specific logo with adjustable sizing.
```
