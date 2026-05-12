#!/bin/bash
# Build script for AsioBridge installer
# Requires: NSIS (makensis), Tauri build output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
INSTALLER_DIR="$PROJECT_DIR/installer"
DIST_DIR="$PROJECT_DIR/app/dist/win-unpacked"

echo "=== AsioBridge Installer Builder ==="

# Check for NSIS
if ! command -v makensis &> /dev/null; then
  echo "ERROR: NSIS (makensis) not found"
  echo "Install from: https://nsis.org/"
  exit 1
fi

# Check for Tauri build output
if [ ! -d "$DIST_DIR" ]; then
  echo "ERROR: Tauri build output not found at $DIST_DIR"
  echo "Run 'pnpm tauri build' first"
  exit 1
fi

# Check for driver files (optional)
DRIVER_DIR="$PROJECT_DIR/driver/asiovadpro/x64/Release"
if [ -d "$DRIVER_DIR" ]; then
  echo "Found driver files in $DRIVER_DIR"
else
  echo "Driver files not found (optional - will build without driver)"
fi

# Copy build output to installer directory
echo "Preparing installer files..."
rm -rf "$INSTALLER_DIR/dist"
cp -r "$DIST_DIR" "$INSTALLER_DIR/dist"

# Copy driver files if available
if [ -d "$DRIVER_DIR" ]; then
  mkdir -p "$INSTALLER_DIR/driver/asiovadpro"
  cp "$DRIVER_DIR/asiovadpro.sys" "$INSTALLER_DIR/driver/asiovadpro/" 2>/dev/null || true
fi

# Build installer
echo "Building installer..."
cd "$INSTALLER_DIR"
makensis -DVERSION="$(grep -oP '(?<=version = ")[^"]*' ../Cargo.toml)" asiobridge.nsi

# Clean up
rm -rf "$INSTALLER_DIR/dist"

echo ""
echo "=== Build Complete ==="
if [ -f "$INSTALLER_DIR/AsioBridge-Setup.exe" ]; then
  echo "Installer: $INSTALLER_DIR/AsioBridge-Setup.exe"
  ls -lh "$INSTALLER_DIR/AsioBridge-Setup.exe"
else
  echo "ERROR: Installer not found"
  exit 1
fi
