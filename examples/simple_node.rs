use avi_p2p::{AviP2p, AviP2pConfig, AviEvent};
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zero-config: Default settings enable mDNS
    let config = AviP2pConfig::default();

    println!("🚀 Starting AVI P2P Node...");
    println!("   Scanning local network for peers (mDNS)...");

    let (node, mut event_rx) = AviP2p::start(config).await?;
    let handle = node.handle();

    // Event Loop
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                AviEvent::Started { local_peer_id, .. } => {
                    println!("✅ Node Online: {}", local_peer_id);
                },
                AviEvent::PeerDiscovered { peer_id } => {
                    // This triggers when mDNS finds someone.
                    // The runtime now Auto-Dials, so a Connected event should follow shortly.
                    println!("🔎 Found Peer: {}", peer_id);
                },
                AviEvent::PeerConnected { peer_id, .. } => {
                    println!("🔗 CONNECTED to {}", peer_id);
                },
                AviEvent::PeerDisconnected { peer_id } => {
                    println!("🔌 Disconnected from {}", peer_id);
                },
                AviEvent::Message { from, topic, data } => {
                    let msg = String::from_utf8_lossy(&data);
                    println!("📩 [{}] {}: {}", topic, from, msg);
                },
                _ => {}
            }
        }
    });

    println!("\nCommands: 'sub <topic>', 'pub <topic> <msg>', 'peers'");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Ok(Some(line)) = stdin.next_line().await {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "sub" if parts.len() > 1 => {
                handle.subscribe(parts[1]).await?;
                println!("Subscribed to '{}'", parts[1]);
            },
            "pub" if parts.len() > 2 => {
                let topic = parts[1];
                let content = parts[2..].join(" ");
                handle.publish(topic, content.into_bytes()).await?;
            },
            "peers" => {
                let peers = handle.connected_peers().await?;
                println!("Connected peers: {:?}", peers);
            },
            "quit" => break,
            _ => println!("Unknown command"),
        }
    }

    node.shutdown().await?;
    Ok(())
}