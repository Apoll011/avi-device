use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;
use crate::{AviP2pHandle, PeerId, StreamId};
use avi_p2p_protocol::{UplinkMessage, DownlinkMessage, MAX_PACKET_SIZE};

pub struct BridgeConfig {
    pub udp_port: u16,
}

struct DeviceSession {
    pub device_id: u64,

    active_streams: HashMap<u8, StreamId>,
}

pub struct EmbeddedBridge {
    socket: Arc<UdpSocket>,
    handle: AviP2pHandle,

    sessions: Arc<Mutex<HashMap<SocketAddr, DeviceSession>>>,
}

impl EmbeddedBridge {
    pub async fn start(handle: AviP2pHandle, config: BridgeConfig) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", config.udp_port);
        let socket = UdpSocket::bind(&addr).await.map_err(|e| e.to_string())?;
        let socket = Arc::new(socket);

        let sessions = Arc::new(Mutex::new(HashMap::new()));

        println!("🌉 Embedded Bridge Listening on UDP {}", config.udp_port);

        // Spawn the UDP listener loop
        tokio::spawn(async move {
            let mut buf = [0u8; MAX_PACKET_SIZE];

            loop {
                // 1. Receive Packet
                let (len, remote_addr) = match socket.recv_from(&mut buf).await {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                // 2. Parse Packet (Zero-copy)
                let packet: Result<UplinkMessage, _> = postcard::from_bytes(&buf[..len]);

                if let Ok(msg) = packet {
                    Self::handle_packet(
                        msg,
                        remote_addr,
                        socket.clone(),
                        handle.clone(),
                        sessions.clone()
                    ).await;
                }
            }
        });

        Ok(())
    }

    async fn handle_packet(
        msg: UplinkMessage<'_>,
        addr: SocketAddr,
        socket: Arc<UdpSocket>,
        handle: AviP2pHandle,
        sessions: Arc<Mutex<HashMap<SocketAddr, DeviceSession>>>,
    ) {
        let mut sessions_lock = sessions.lock().await;

        match msg {
            // --- CONNECT ---
            UplinkMessage::Hello { device_id } => {
                println!("🌉 New Device Connected: ID {} at {}", device_id, addr);

                // Create Session
                sessions_lock.insert(addr, DeviceSession {
                    device_id,
                    active_streams: HashMap::new(),
                });

                // Send Welcome
                let welcome = DownlinkMessage::Welcome;
                let mut tx_buf = [0u8; 64];
                if let Ok(data) = postcard::to_slice(&welcome, &mut tx_buf) {
                    let _ = socket.send_to(data, addr).await;
                }
            },

            // --- START STREAM ---
            UplinkMessage::StreamStart { local_stream_id, target_peer_id } => {
                if let Some(session) = sessions_lock.get_mut(&addr) {

                    // 1. Resolve Target
                    // If empty string, we currently fail (or you could pick a default peer)
                    if target_peer_id.is_empty() {
                        println!("⚠️ Device requested stream with no target.");
                        return;
                    }

                    // 2. Request P2P Stream via Handle
                    let peer_id = PeerId::new(target_peer_id);
                    println!("🌉 Bridging Stream {} -> Mesh Peer {}", local_stream_id, peer_id);

                    match handle.request_stream(peer_id).await {
                        Ok(mesh_stream_id) => {
                            // 3. Map the IDs
                            session.active_streams.insert(local_stream_id, mesh_stream_id);
                        },
                        Err(e) => eprintln!("❌ Bridge Failed to open mesh stream: {}", e),
                    }
                }
            },

            // --- DATA ---
            UplinkMessage::StreamData { local_stream_id, data } => {
                if let Some(session) = sessions_lock.get(&addr) {
                    // 1. Find the Mesh ID
                    if let Some(mesh_id) = session.active_streams.get(&local_stream_id) {
                        // 2. Forward to Mesh
                        // Note: to_vec() allocates, but necessary to cross async boundary
                        let _ = handle.send_stream_data(*mesh_id, data.to_vec()).await;
                    }
                }
            },

            // --- CLOSE ---
            UplinkMessage::StreamClose { local_stream_id } => {
                if let Some(session) = sessions_lock.get_mut(&addr) {
                    if let Some(mesh_id) = session.active_streams.remove(&local_stream_id) {
                        let _ = handle.close_stream(mesh_id).await;
                        println!("🌉 Closed Bridged Stream");
                    }
                }
            }

            UplinkMessage::ButtonPress { button_id, press_type } => {
                if let Some(session) = sessions_lock.get(&addr) {
                    let dev_id = session.device_id;

                    // Example: "avi/home/device_1234/button"
                    let topic = format!("avi/home/device_{}/button", dev_id);

                    // 2. Construct JSON Payload
                    let payload = json!({
                        "button_id": button_id,
                        "type": format!("{:?}", press_type),
                        "ts": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs()
                    });

                    // 3. Publish to Mesh
                    println!("🌉 [Bridge] Button {} ({:?}) -> {}", button_id, press_type, topic);
                    let _ = handle.publish(&topic, serde_json::to_vec(&payload).unwrap()).await;
                }
            },

            UplinkMessage::SensorUpdate { sensor_name, data } => {
                if let Some(session) = sessions_lock.get(&addr) {
                    let dev_id = session.device_id;

                    // Example: "avi/home/device_1234/sensor/temp_kitchen"
                    let topic = format!("avi/home/device_{}/sensor/{}", dev_id, sensor_name);

                    // 2. Construct JSON Payload
                    // We interpret the Enum into a clean JSON value
                    let val = match data {
                        avi_p2p_protocol::SensorValue::Temperature(v) => json!(v),
                        avi_p2p_protocol::SensorValue::Humidity(v) => json!(v),
                        avi_p2p_protocol::SensorValue::Battery(v) => json!(v),
                        avi_p2p_protocol::SensorValue::Status(v) => json!(v),
                        avi_p2p_protocol::SensorValue::Raw(v) => json!(v),
                    };

                    let payload = json!({
                        "value": val,
                        "unit": match data {
                            avi_p2p_protocol::SensorValue::Temperature(_) => "C",
                            avi_p2p_protocol::SensorValue::Humidity(_) => "%",
                            _ => ""
                        },
                        "ts": std::time::SystemTime::now()
                           .duration_since(std::time::UNIX_EPOCH)
                           .unwrap_or_default().as_secs()
                    });

                    // 3. Publish to Mesh
                    println!("🌉 [Bridge] Sensor {} -> {}", sensor_name, topic);
                    let _ = handle.publish(&topic, serde_json::to_vec(&payload).unwrap()).await;

                    // Optional: Update the CRDT Context automatically?
                    // You could also call handle.update_context(...) here to sync state globally!
                }
            }
        }
    }
}