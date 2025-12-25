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

Add `avi-p2p` to your `Cargo.toml`:

```toml
[dependencies]
avi-p2p = { git = "https://github.com/apoll011/avi-p2p" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

### Basic Device Setup

```rust
use avi_p2p::device::{AviDevice, AviDeviceConfig, AviDeviceType};
use avi_p2p::DeviceCapabilities;

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

    println!("🚀 AVI Device is running!");
    
    // Keep the application alive
    tokio::signal::ctrl_c().await.ok();
    Ok(())
}
```

---

## 📖 Feature Examples

### 1. 🧠 Distributed Unified Context (Shared State)

Maintain a shared state across all devices. Updates are automatically synchronized.

```rust
use serde_json::json;

// Update a specific nested path (creates objects if they don't exist)
device.update_ctx("avi.device.audio.volume", json!(80)).await.ok();

// Retrieve a value from the context
if let Ok(volume) = device.get_ctx("avi.device.audio.volume").await {
    println!("Current Volume: {}", volume);
}

// Update a whole object
device.update_ctx("avi.device.status", json!({
    "online": true,
    "mode": "active"
})).await.ok();
```

### 2. 📢 Pub/Sub Messaging

Topic-based communication using Gossipsub for efficient broadcasting.

```rust
// Subscribe to a topic
device.subscribe("system/alerts", |from, topic, data| {
    let msg = String::from_utf8_lossy(&data);
    println!("Received alert from {}: {}", from, msg);
}).await.ok();

// Publish to a topic
let message = "Low battery warning!";
device.publish("system/alerts", message.as_bytes().to_vec()).await.ok();
```

### 3. 🎙️ Real-time Data Streams

Open direct, managed streams for high-frequency data like audio or logs.

#### Implementing a Stream Handler
```rust
use avi_p2p::{StreamHandler, StreamContext, StreamHandlerFactory, PeerId, StreamId, StreamCloseReason};
use async_trait::async_trait;

pub struct MyStreamHandler;

#[async_trait]
impl StreamHandler for MyStreamHandler {
    async fn on_accepted(&mut self, ctx: &StreamContext) {
        println!("Stream {} established with {}", ctx.stream_id, ctx.peer_id);
    }

    async fn on_data(&mut self, ctx: &StreamContext, data: Vec<u8>) {
        println!("Received {} bytes", data.len());
    }
    
    async fn on_rejected(&mut self, _peer: PeerId, _id: StreamId, reason: String) {
        println!("Stream rejected: {}", reason);
    }

    async fn on_closed(&mut self, _peer: PeerId, _id: StreamId, _reason: StreamCloseReason) {
        println!("Stream closed");
    }
}

pub struct MyStreamFactory;
#[async_trait]
impl StreamHandlerFactory for MyStreamFactory {
    async fn create_handler(&self) -> Box<dyn StreamHandler> {
        Box::new(MyStreamHandler)
    }
}
```

#### Registering and Requesting Streams
```rust
// Register the handler to handle incoming "audio" stream requests
device.register_stream_handler("audio".to_string(), MyStreamFactory).await;

// Request a stream to a specific peer
let target_peer = PeerId::from_str("...").unwrap();
let stream_id = device.request_stream(target_peer, "audio".to_string()).await.unwrap();

// Send data through the stream
device.send_stream_data(stream_id, vec![0, 1, 2, 3]).await.ok();
```

### 4. 🔍 Capability-Based Querying

Find devices in the network based on their hardware or software capabilities.

```rust
use avi_p2p::DeviceQuery;
use avi_p2p::capability::{CapabilityBuilder, SensorCapability};

// 1. Defining capabilities at startup
let caps = CapabilityBuilder::new()
    .sensor("microphone", SensorCapability::Microphone {
        present: true,
        array_size: 4,
        sampling_rate_khz: 48,
        max_spl_db: 120,
    })
    .build();

// 2. Querying the network for devices with a microphone
let query = DeviceQuery::new()
    .sensor("microphone", |s| {
        matches!(s, SensorCapability::Microphone { present: true, .. })
    });

let results = device.execute_query(query).await.unwrap();
println!("Found devices with microphone: {:?}", results);
```

---

## 🛠️ Advanced Capability Builder

The `CapabilityBuilder` allows you to define complex device profiles:

```rust
let caps = CapabilityBuilder::new()
    .compute(ComputeCapability { ... })
    .connectivity("wifi", ConnectivityCapability::Wifi { ... })
    .display(DisplayCapability { ... })
    .extended("custom_feature", ExtendedCapability::Boolean(true))
    .build();
```

---

## 📖 Examples Directory

For more detailed implementations, check the `examples/` folder:

*   **[`device_context.rs`](./examples/device_context.rs)**: Distributed context deep-dive.
*   **[`device_pubsub.rs`](./examples/device_pubsub.rs)**: Efficient messaging patterns.
*   **[`device_stream.rs`](./examples/device_stream.rs)**: High-performance data streaming.
*   **[`device_query.rs`](./examples/device_query.rs)**: Peer discovery and filtering.
*   **[`audio_chat.rs`](./examples/audio_chat.rs)**: Real-world example of audio streaming.

---

## 🏗️ Architecture

The framework is layered for flexibility:

1.  **`p2p` crate**: The core networking engine.
    -   Built on `libp2p` (Gossipsub, Kademlia, mDNS).
    -   Implements CRDT-like context synchronization.
2.  **`avi-p2p` (root crate)**: The high-level developer API.
    -   Managed `StreamDispatcher`.
    -   Path-based context manipulation.
    -   Simplified device lifecycle.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
