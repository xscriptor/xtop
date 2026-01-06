#!/bin/bash

# xtop installer
# Installs xtop by building from source (requires Cargo) or checking for pre-built binaries (future)

set -e

APP_NAME="xtop"
REPO_URL="https://github.com/xscriptordev/xtop.git" # Replace with actual repo URL if known, or generic
INSTALL_DIR="/usr/local/bin"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing $APP_NAME...${NC}"

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo (Rust) is not installed.${NC}"
    echo "Please install Rust first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
echo "Working in $TEMP_DIR"

# Clone or download source
# Since this script might be run remotely, we need to get the source.
# Assuming we can git clone for now.
if command -v git &> /dev/null; then
    echo "Cloning repository..."
    git clone --depth 1 "$REPO_URL" "$TEMP_DIR/$APP_NAME" || {
        # Fallback if repo URL is placeholder or fails
        echo -e "${RED}Failed to clone repository. If this is a local script, run it from the project root.${NC}"
        # If running locally, use current dir
        if [ -f "Cargo.toml" ]; then
            echo "Detected local source, building from current directory..."
            cp -r . "$TEMP_DIR/$APP_NAME"
        else
             exit 1
        fi
    }
else
    echo -e "${RED}Error: git is not installed.${NC}"
    exit 1
fi

cd "$TEMP_DIR/$APP_NAME"

echo "Building $APP_NAME..."
cargo build --release

echo "Installing binary to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    cp "target/release/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
else
    echo "Sudo permissions required to install to $INSTALL_DIR"
    sudo cp "target/release/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
fi

# Cleanup
rm -rf "$TEMP_DIR"

echo -e "${GREEN}$APP_NAME installed successfully!${NC}"
echo "Run it with: $APP_NAME"
