use avi_p2p::{AviP2p, AviP2pConfig, AviEvent};
use tokio::io::{self, AsyncBufReadExt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Zero-Config Startup
    let config = AviP2pConfig::default();
    println!("🚀 Starting Smart Home Node...");

    let (node, mut event_rx) = AviP2p::start(config).await?;
    let handle = node.handle();

    // 2. Initialize Default Context
    // We set some initial "Smart Home" values for this device
    let initial_state = json!({
        "device": {
            "name": "Unknown Device",
            "type": "generic",
            "battery_pct": 100
        },
        "app": {
            "status": "idle"
        }
    });
    handle.update_context(initial_state).await?;

    // 3. Event Loop (Watch for updates from other devices)
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                AviEvent::Started { local_peer_id, .. } => {
                    println!("✅ Node Online: {}", local_peer_id);
                },
                AviEvent::PeerConnected { peer_id, .. } => {
                    println!("🔗 Device Connected: {}", peer_id);
                },
                // --- CONTEXT UPDATE EVENT ---
                AviEvent::ContextUpdated { peer_id, context } => {
                    // This fires whenever a peer changes their state
                    println!("\n🔄 UPDATE from {}:", peer_id);
                    // Pretty-print the updated JSON
                    if let Ok(pretty) = serde_json::to_string_pretty(&context) {
                        println!("{}", pretty);
                    }
                },
                _ => {}
            }
        }
    });

    // 4. Interactive Command Loop
    println!("\nCommands:");
    println!("  name <name>       -> Set device name (e.g., 'Kitchen Speaker')");
    println!("  vol <0-100>       -> Update volume");
    println!("  status <status>   -> Update app status (e.g., 'playing', 'paused')");
    println!("  batt <0-100>      -> Update battery level");
    println!("  get <peer_id>     -> Fetch full context of a peer");
    println!("  me                -> Show my full context");
    println!("  quit              -> Exit");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Ok(Some(line)) = stdin.next_line().await {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "name" if parts.len() > 1 => {
                let name = parts[1..].join(" ");
                // Partial update: Only changes "device.name", keeps other fields intact
                let patch = json!({
                    "device": { "name": name }
                });
                handle.update_context(patch).await?;
                println!("✅ Name updated.");
            },
            "vol" if parts.len() > 1 => {
                if let Ok(vol) = parts[1].parse::<u8>() {
                    let patch = json!({
                        "device": { "audio": { "volume": vol } }
                    });
                    handle.update_context(patch).await?;
                    println!("✅ Volume set to {}", vol);
                }
            },
            "batt" if parts.len() > 1 => {
                if let Ok(pct) = parts[1].parse::<u8>() {
                    let patch = json!({
                        "device": { "battery_pct": pct }
                    });
                    handle.update_context(patch).await?;
                    println!("✅ Battery set to {}%", pct);
                }
            },
            "status" if parts.len() > 1 => {
                let status = parts[1];
                let patch = json!({
                    "app": { "status": status }
                });
                handle.update_context(patch).await?;
                println!("✅ Status set to '{}'", status);
            },
            "get" if parts.len() > 1 => {
                let target = avi_p2p::PeerId::new(parts[1]);
                match handle.get_context(Some(target)).await {
                    Ok(ctx) => println!("📄 Context:\n{}", serde_json::to_string_pretty(&ctx).unwrap()),
                    Err(e) => eprintln!("❌ Error: {}", e),
                }
            },
            "me" => {
                match handle.get_context(None).await {
                    Ok(ctx) => println!("🏠 My Context:\n{}", serde_json::to_string_pretty(&ctx).unwrap()),
                    Err(e) => eprintln!("❌ Error: {}", e),
                }
            },
            "quit" => break,
            _ => println!("Unknown command"),
        }
    }

    node.shutdown().await?;
    Ok(())
}