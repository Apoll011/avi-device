use avi_p2p_embedded::{AviEmbedded, AviEmbeddedConfig, UdpClient};
use avi_p2p_protocol::{PressType, SensorValue};
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::io;

// --- MOCK HARDWARE LAYER ---
// This wrapper adapts a PC UDP socket to the trait expected by the embedded library.
// On an actual ESP32, you would wrap the 'embassy-net' socket here.
struct PcSocket {
    inner: UdpSocket,
    target: SocketAddr,
}

impl PcSocket {
    async fn new(bind_addr: &str, target_addr: &str) -> io::Result<Self> {
        let sock = UdpSocket::bind(bind_addr).await?;
        let target = target_addr.parse().unwrap();
        Ok(Self { inner: sock, target })
    }
}

// We implement the trait defined in your embedded library
impl UdpClient for PcSocket {
    type Error = io::Error;

    async fn send(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.inner.send_to(buf, self.target).await.map(|_| ())
    }

    async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let (len, _) = self.inner.recv_from(buf).await?;
        Ok(len)
    }
}
// ---------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 🔌 SIMULATED EMBEDDED DEVICE (ID: 5555) ---");

    // 1. Setup Network (Mocking Hardware)
    // Bind to a random port, target localhost:8888 (The Gateway)
    let socket = PcSocket::new("127.0.0.1:0", "127.0.0.1:8888").await?;

    // 2. Setup Embedded Library
    let mut scratch_buffer = [0u8; 1024]; // Stack buffer in real embedded
    let config = AviEmbeddedConfig { device_id: 5555 };

    let mut mcu = AviEmbedded::new(socket, config, &mut scratch_buffer);

    // 3. Connect Phase
    println!("⏳ Connecting to Gateway...");
    if let Ok(_) = mcu.connect().await {
        println!("✅ Handshake Complete! Connected to Bridge.");
    } else {
        eprintln!("❌ Failed to connect (is the gateway running?)");
        return Ok(());
    }

    // 4. Main Device Loop (Simulate User Interaction)
    let mut counter = 0;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Simulate a Double Click Button
        println!("👉 [MCU] User Double-Clicked Button 1");
        mcu.button_pressed(1, PressType::Double).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Simulate a Temperature Sensor Update
        let temp = 20.0 + (counter as f32 * 0.5);
        println!("🌡️  [MCU] Reading Sensor: {} C", temp);

        mcu.update_sensor("kitchen_temp", SensorValue::Temperature(temp))
            .await
            .unwrap();

        counter += 1;
        println!("--------------------------------");
    }
}