#!/bin/bash

# About This Linux - Installation Script
# Version 0.3.0
# Author: Kamil 'Novik' Nowicki <kamil.nowicki@h4b.uk>

set -e

BINARY_NAME="about-this-linux"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.local/share/about-this-linux"

echo "About This Linux v0.3.0 - Installation Script"
echo "=============================================="
echo "Author: Kamil 'Novik' Nowicki <kamil.nowicki@h4b.uk>"
echo

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "Please do not run this script as root."
    echo "It will ask for sudo password when needed."
    exit 1
fi

# Check dependencies
echo "Checking dependencies..."

check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        echo "ERROR: $1 is not installed."
        echo "Please install $1 and try again."
        echo
        echo "On Arch Linux: sudo pacman -S $2"
        echo "On Ubuntu/Debian: sudo apt install $3"
        exit 1
    else
        echo "✓ $1 found"
    fi
}

check_dependency "cargo" "rust" "cargo"
check_dependency "fastfetch" "fastfetch" "fastfetch"
check_dependency "dmidecode" "dmidecode" "dmidecode"
check_dependency "lsblk" "util-linux" "util-linux"

echo

# Check GTK4 development libraries
echo "Checking GTK4 development libraries..."
if ! pkg-config --exists gtk4; then
    echo "ERROR: GTK4 development libraries not found."
    echo "Please install GTK4 development packages:"
    echo
    echo "On Arch Linux: sudo pacman -S gtk4"
    echo "On Ubuntu/Debian: sudo apt install libgtk-4-dev"
    exit 1
else
    echo "✓ GTK4 development libraries found"
fi

echo

# Build the application
echo "Building the application..."
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Cargo.toml not found. Are you in the correct directory?"
    exit 1
fi

cargo build --release

if [ $? -ne 0 ]; then
    echo "ERROR: Build failed. Please check the error messages above."
    exit 1
fi

echo "✓ Build completed successfully"
echo

# Install the binary
echo "Installing the application..."
if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo "ERROR: Binary not found at target/release/$BINARY_NAME"
    exit 1
fi

sudo cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✓ Binary installed to $INSTALL_DIR/$BINARY_NAME"

# Create config directory
echo "Creating config directory..."
mkdir -p "$CONFIG_DIR"
echo "✓ Config directory created at $CONFIG_DIR"

# Copy default image if it exists
if [ -f "tux-logo.png" ]; then
    cp "tux-logo.png" "$CONFIG_DIR/"
    echo "✓ Default logo copied to config directory"
fi

echo
echo "Installation completed successfully!"
echo
echo "You can now run the application with:"
echo "  $BINARY_NAME"
echo
echo "Or force the configuration wizard with:"
echo "  $BINARY_NAME --configure"
echo
echo "For help:"
echo "  $BINARY_NAME --help"
echo
echo "Note: On first run, the application will automatically show"
echo "the configuration wizard if no config file is found."
echo
echo "Enjoy About This Linux!"
