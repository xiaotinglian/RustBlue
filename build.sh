#!/bin/bash

# Build script for rustblue

set -e

echo "Building rustblue..."

# Build in release mode
cargo build --release

echo "Build completed successfully!"

# Copy binary to target location (optional)
if [ "$1" = "install" ]; then
    echo "Installing rustblue..."
    sudo cp target/release/rustblue /usr/local/bin/
    sudo cp data/org.rustblue.Manager.desktop /usr/share/applications/
    echo "Installation completed!"
fi

echo "You can now run: ./target/release/blueman-rs"
