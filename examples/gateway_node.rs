use avi_p2p::{AviP2p, AviP2pConfig, AviEvent};
use avi_p2p::bridge::{EmbeddedBridge, BridgeConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 📡 AVI P2P GATEWAY & MESH MONITOR ---");

    // ==========================================
    // NODE A: THE GATEWAY (Bridge)
    // ==========================================
    let mut config_a = AviP2pConfig::default();
    config_a.node_name = "gateway-hub".to_string();
    config_a.listen_port = 0; // Random TCP port

    let (gateway_node, mut gateway_events) = AviP2p::start(config_a).await?;
    let gateway_handle = gateway_node.handle();

    // Start the UDP Bridge on Port 8888
    EmbeddedBridge::start(
        gateway_handle.clone(),
        BridgeConfig { udp_port: 8888 }
    ).await.unwrap();

    println!("✅ Gateway started.");
    println!("🌉 Bridge listening on UDP 0.0.0.0:8888");

    // Spin up a task to drain gateway events so the loop doesn't block
    tokio::spawn(async move {
        while let Some(_) = gateway_events.recv().await {}
    });

    // ==========================================
    // NODE B: THE MONITOR (Dashboard/Subscriber)
    // ==========================================
    // This represents a UI or another speaker on the network
    // watching for sensor data.
    println!("🚀 Starting Monitor Node (to prove P2P works)...");

    let mut config_b = AviP2pConfig::default();
    config_b.node_name = "dashboard-ui".to_string();

    // Connect Node B to Node A (Gateway) via mDNS auto-discovery
    let (monitor_node, mut monitor_events) = AviP2p::start(config_b).await?;
    let monitor_handle = monitor_node.handle();

    // Subscribe to the specific topics the Bridge publishes to
    // Note: In real app, you might use wildcard logic if supported or subscribe to specific IDs
    // For this demo, we assume we know the device ID is 5555
    sleep(Duration::from_secs(2)).await; // Wait for mesh connection
    monitor_handle.subscribe("avi/home/device_5555/button").await?;
    monitor_handle.subscribe("avi/home/device_5555/sensor/kitchen_temp").await?;

    println!("👀 Monitor subscribed to device_5555 topics.");
    println!("   Waiting for Embedded data...\n");

    // ==========================================
    // MONITOR EVENT LOOP
    // ==========================================
    while let Some(event) = monitor_events.recv().await {
        match event {
            AviEvent::Message { from, topic, data } => {
                let json_str = String::from_utf8_lossy(&data);
                println!(
                    "⚡ [MESH EVENT] Topic: {}\n   From Gateway: {}\n   Payload: {}\n",
                    topic, from, json_str
                );
            }
            AviEvent::PeerConnected { peer_id, .. } => {
                println!("🔗 Monitor connected to peer: {}", peer_id);
            }
            _ => {}
        }
    }

    Ok(())
}