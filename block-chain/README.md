# Block Chain Node Monitor

This project includes an egui-based GUI to display connected node count.

## Features

- Display current number of connected peer nodes
- Show list of connected node addresses
- Auto-refresh interface

## Dependencies

- Rust 1.70+
- tokio
- egui 0.28
- eframe 0.28

## Building

```bash
cd block-chain
cargo build
```

## Running

```bash
cargo run
```

Note: On macOS, the GUI requires running on the main thread. The current implementation spawns GUI in a separate thread, which may show an error on macOS. For full GUI support on macOS, you can modify the code to run GUI on the main thread using `cocoa` directly.

## Configuration

Configuration is loaded from `config/config.json`:

```json
{
    "server": {
        "host": "127.0.0.1",
        "port": 10100
    }
}
```

## Architecture

- `main.rs` - Main application with async blockchain server and GUI integration
- `gui.rs` - egui-based GUI for node monitoring
- `block.rs`, `transaction.rs` - Blockchain data structures
- `async_task.rs` - Async task utilities

## Node Connection

The blockchain server listens on `127.0.0.1:10100` for incoming peer connections. Each new connection:
1. Receives blockchain data from the new peer
2. Updates the peer count in the GUI
3. Stores the peer's write stream for future broadcasts

## TODO

Integrate Dash https://dashpay.github.io/docs-platform/dash_sdk