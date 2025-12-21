# wifi-p2p-rs

A Rust library for Wi-Fi Direct (P2P) operations on Linux systems, providing an asynchronous API similar to Android's Wi-Fi P2P framework. This library communicates with `wpa_supplicant` via D-Bus to perform Wi-Fi Direct operations.

## Overview

`wifi-p2p-rs` is a Rust library that enables Wi-Fi Direct (Peer-to-Peer) operations on Linux systems through `wpa_supplicant`'s D-Bus interface. The library provides an asynchronous API that mirrors Android's Wi-Fi P2P framework concepts, making it familiar for developers who have worked with Android's Wi-Fi Direct APIs.

## Features

- **Asynchronous API**: Built on Tokio for efficient async operations
- **Android-like interface**: Familiar API design for Android developers
- **Event-driven architecture**: Subscribe to P2P events via broadcast channels
- **Platform-specific backend**: Currently supports Linux via `wpa_supplicant`
- **Type-safe error handling**: Comprehensive error types with `thiserror`
- **D-Bus integration**: Communicates with `wpa_supplicant` via system D-Bus

## Prerequisites

- Linux system with `wpa_supplicant` running (with P2P support)
- D-Bus system bus accessible
- Rust toolchain (edition 2024)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wifi-p2p-rs = {git="https://github.com/leset0ng/wifi-p2p-rs.git"}
```

## Quick Start

```rust
use wifi_p2p_rs::{P2pEvent, WifiP2pManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace "wlan0" with your Wi-Fi interface
    let manager = WifiP2pManager::new("wlan0").await?;
    let channel = manager.initialize();

    // Subscribe to P2P events
    let mut events = channel.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                P2pEvent::DiscoveryStarted => {
                    println!("P2P discovery started");
                }
                P2pEvent::DiscoveryStopped => {
                    println!("P2P discovery stopped");
                }
                P2pEvent::GroupCreated => {
                    println!("P2P group created");
                }
                P2pEvent::Connected(addr) => {
                    println!("Connected to peer {addr}");
                }
                P2pEvent::PeerFound(device) => {
                    println!(
                        "Peer found: {} ({:?})",
                        device.mac_address, device.device_name
                    );
                }
            }
        }
    });

    // Start peer discovery
    let action = channel.discover_peers().await?;
    action.await??;

    Ok(())
}
```

## API Reference

### Core Types

- `WifiP2pManager`: Main entry point for creating P2P sessions
- `WifiP2pChannel`: Command channel for P2P operations
- `P2pEvent`: Enum of possible P2P events
- `P2pDevice`: Represents a discovered peer device
- `P2pError`: Comprehensive error type for all P2P operations

### Key Methods

#### `WifiP2pManager`
- `new(interface_name: &str)`: Creates a new manager for the specified interface
- `initialize()`: Sets up the command channel and background worker
- `connection()`: Returns the raw D-Bus connection for advanced use

#### `WifiP2pChannel`
- `subscribe_events()`: Returns a broadcast receiver for P2P events
- `discover_peers()`: Starts peer discovery scan
- `stop_discovery()`: Stops ongoing discovery
- `connect(device_address: String)`: Connects to a peer device
- `create_group()`: Creates a P2P group

### Events

The library emits the following events via `P2pEvent`:

- `DiscoveryStarted`: Peer discovery scan has started
- `DiscoveryStopped`: Peer discovery has stopped
- `GroupCreated`: A P2P group has been created
- `Connected(String)`: Connected to a peer (contains MAC address)
- `PeerFound(P2pDevice)`: A peer device has been discovered

## Architecture

The library follows a layered architecture:

1. **Manager Layer** (`WifiP2pManager`): High-level API entry point
2. **Channel Layer** (`WifiP2pChannel`): Command/event communication
3. **Backend Layer** (`P2pBackend`): Platform-specific implementation
4. **D-Bus Layer**: Direct communication with `wpa_supplicant`

### Backend Abstraction

The library uses a trait-based backend system:

```rust
pub trait P2pBackend: Send + Sync {
    fn discover_peers(&self) -> P2pFuture<'_, ()>;
    fn stop_discovery(&self) -> P2pFuture<'_, ()>;
    fn connect(&self, device_address: String) -> P2pFuture<'_, ()>;
    fn create_group(&self) -> P2pFuture<'_, ()>;
}
```

Currently, only Linux backend is implemented via `wpa_supplicant`'s D-Bus API.

## Error Handling

The library uses `thiserror` for comprehensive error types:

```rust
pub enum P2pError {
    DBus(#[from] zbus::Error),
    ZVariant(#[from] zbus::zvariant::Error),
    ChannelClosed(String),
    InvalidInterface(String),
    Backend(String),
}
```

All async methods return `Result<T, P2pError>`.

## Dependencies

- `tokio`: Async runtime
- `zbus`: D-Bus communication
- `thiserror`: Error handling

## Platform Support

Currently supported:
- **Linux**: Via `wpa_supplicant` D-Bus interface

Planned support:
- Other platforms (contributions welcome!)

## Limitations

- Requires `wpa_supplicant` with P2P support enabled
- Currently only supports basic P2P operations
- Event system is a simplified version of Android's broadcast intents

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the terms you choose (specify in your project).

## Disclaimer

**Note**: This library is currently in early development and may have limitations. Test thoroughly before production use.

## Getting Help

- Review the source code for detailed implementation
- Check the example usage in `src/example.rs`
- File issues on GitHub for bugs or feature requests
