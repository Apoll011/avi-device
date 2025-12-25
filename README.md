# AVI P2P & Device Framework

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

A powerful, distributed networking framework built on Libp2p for AVI devices. It provides a high-level abstraction for peer-to-peer communication, distributed state management, and real-time data streaming.

---

## 🌟 Key Features

*   **🧠 Unified Distributed Context:** A shared, global state across all devices in the mesh. Automatic synchronization and conflict resolution.
*   **🔗 Seamless Connectivity:** Automatic peer discovery via mDNS, Kademlia DHT, and bootstrap nodes.
*   **📢 High-Level Pub/Sub:** Topic-based messaging using Gossipsub for efficient broadcasting.
*   **🎙️ Managed Data Streams:** Trait-based stream dispatcher for real-time data like audio, logs, or sensor feeds.
*   **🔍 Capability-Based Querying:** Find devices based on what they can do (e.g., "find all devices with a display").
*   **🛡️ Robust Sync Logic:** Oldest-wins conflict resolution and deep-merge strategy for eventual consistency.

---

## 🚀 Quick Start

### 1. Installation

Add `avi-device` to your `Cargo.toml`:

```toml
[dependencies]
avi-device = { git = "https://github.com/apoll011/avi-device" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

### 2. Basic Usage

```rust
use avi_device::device::{AviDevice, AviDeviceConfig, AviDeviceType};
use avi_device::DeviceCapabilities;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), String> {
    // 1. Configure the device
    let config = AviDeviceConfig {
        node_name: "smart-speaker-01".to_string(),
        device_type: AviDeviceType::NODE,
        capabilities: DeviceCapabilities::default(),
    };

    // 2. Initialize the device
    let device = AviDevice::new(config).await?;

    // 3. Start the event loop in a background task
    let device_clone = device.clone();
    tokio::spawn(async move {
        device_clone.start_event_loop().await;
    });

    // 4. Update shared context
    device.update_ctx("avi.device.status", json!({"playing": true, "volume": 80})).await.ok();

    // 5. Subscribe to messages
    device.subscribe("alerts", |from, topic, data| {
        println!("Received alert from {}: {}", from, String::from_utf8_lossy(&data));
    }).await.ok();

    Ok(())
}
```

---

## 🧠 Distributed Unified Context

One of the core features of the AVI framework is the **Unified Context**. Every device in the network maintains a local copy of a shared state that eventually converges across the entire mesh.

### How it works:
-   **Deep Merge:** When receiving updates, the system performs a deep merge of JSON objects. If a peer has keys that you don't have, they are added to your local state.
-   **Conflict Resolution:** If two devices update the same key concurrently, the "oldest wins" strategy is used based on the update timestamp.
-   **Automatic Sync:** When a new device connects to the network, it automatically exchanges context with its peers to catch up with the latest state.
-   **Path-Based Updates:** You can update specific nested paths (e.g., `device.audio.volume`) without overwriting the entire object.

---

## 📖 Feature Examples

Explore the `examples/` directory for detailed implementations:

*   **[`device_context.rs`](./examples/device_context.rs)**: Deep dive into distributed context updates and retrieval.
*   **[`device_pubsub.rs`](./examples/device_pubsub.rs)**: Standard topic-based communication.
*   **[`device_stream.rs`](./examples/device_stream.rs)**: Setting up managed data streams with custom handlers.
*   **[`device_query.rs`](./examples/device_query.rs)**: Searching the network for specific device capabilities.
*   **[`device_chat.rs`](./examples/device_chat.rs)**: A full CLI application combining all features.

---

## 📚 API Reference

### `AviDevice` (High-Level API)
| Method | Description |
| :--- | :--- |
| `new(config)` | Creates and starts a new AVI device. |
| `update_ctx(path, value)` | Updates a value in the unified context at the specified path. |
| `get_ctx(path)` | Retrieves a value (or sub-tree) from the context. |
| `publish(topic, data)` | Broadcasts raw data to a specific topic. |
| `subscribe(topic, handler)` | Registers a callback for messages on a topic. |
| `request_stream(peer_id, reason)` | Opens a direct data stream to a peer. |
| `register_stream_handler(...)` | Defines how to process incoming stream requests. |
| `execute_query(query)` | Finds peers matching specific capabilities. |

---

## 🏗️ Architecture

The framework is layered for flexibility:

1.  **`p2p` crate**: The core engine.
    -   Uses `libp2p` with `Gossipsub`, `Kademlia`, `Identify`, and `mDNS`.
    -   Implements `AviContext` with CRDT-like merge logic.
    -   Provides a Request-Response based streaming protocol.
2.  **`avi-device` (root crate)**: The developer-friendly wrapper.
    -   Handles the `StreamDispatcher` for managing multiple concurrent streams.
    -   Provides the path-based context API.
    -   Manages the high-level event loop.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
