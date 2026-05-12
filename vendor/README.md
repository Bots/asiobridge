# Vendor SDKs for AsioBridge

These SDKs require accepting Steinberg's EULA before downloading.

## ASIO SDK

1. Go to https://www.steinberg.net/asiosdk/
2. Accept the EULA
3. Download `asio367.zip` (or latest)
4. Extract to `vendor/asio-sdk/`

Expected structure:
```
vendor/asio-sdk/
├── common/
├── host/
├── asiosys/
└── asiodrvr.h
```

## VST3 SDK

1. Go to https://www.steinberg.net/vst3-devcenter/
2. Accept the EULA
3. Download the latest VST3 SDK
4. Extract to `vendor/vst3-sdk/`

Expected structure:
```
vendor/vst3-sdk/
├── public/
│   ├── steinberg/
│   │   ├── base/
│   │   ├── audio/
│   │   └── plugintypes/
│   └── vst/
└── plugins/
```

## After Downloading

Run `cargo build` to compile the FFI bindings.
