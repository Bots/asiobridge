# Build configuration for AsioBridge Virtual Audio Driver
# Requires Windows Driver Kit (WDK) to build

# Target platforms
TARGET_PLATFORMS = win10 win11

# Driver settings
DRIVER_NAME = asiovadpro
DRIVER_TYPE = kmdf
DRIVER_VERSION = 1.0.0

# Build targets
.PHONY: all clean x86 x64 arm64

all: x64

x64:
	@echo "Building $(DRIVER_NAME) for x64..."
	@if exist "C:\Program Files\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" ( \
		echo "WDK found, building..." \
	) || ( \
		echo "ERROR: WDK not found. Please install Windows Driver Kit."; \
		echo "Download from: https://learn.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk"; \
		exit 1; \
	)

clean:
	@echo "Cleaning build artifacts..."
	@if exist "x64\Release" rmdir /s /q "x64\Release"
	@if exist "x64\Release" rmdir /s /q "x64"

# Install the driver (requires admin)
install:
	@echo "Installing $(DRIVER_NAME) driver..."
	@echo "This requires administrator privileges"
	@echo "Run: pnpx install-driver"

# Sign the driver (requires EV certificate)
sign:
	@echo "Signing $(DRIVER_NAME) driver..."
	@echo "Requires EV code signing certificate"
	@echo "Run: signtool sign /v /fd SHA256 /sha1 <CERT_THUMBPRINT> x64\Release\$(DRIVER_NAME).sys"

include $(WINDDK)\build\global.make
