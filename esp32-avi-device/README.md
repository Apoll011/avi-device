# ESP32 AVI Device

This is a ready-to-build ESP32 project for the AVI P2P embedded device.

## Prerequisites

```bash
# Install ESP toolchain
cargo install espup espflash
espup install
source ~/export-esp.sh
```

## Configuration

Edit `src/main.rs` and update:

1. **WiFi credentials** (around line 58):
```rust
let wifi_config = WifiConfig {
    ssid: "YourNetworkName",    // <- Change this
    password: "YourPassword",    // <- Change this
};
```

2. **Gateway IP address** (around line 74):
```rust
let socket = match EspUdpSocket::new([192, 168, 1, 100], 8888) {
    //                                 ^^^^^^^^^^^^^^^^^^^
    //                                 Change to your gateway IP
```

3. **Device ID** (around line 80):
```rust
let config = AviEmbeddedConfig { device_id: 1234 };  // <- Change if needed
```

## Building

```bash
# Build
cargo build --release

# Or build and flash in one command
cargo run --release
```

## Flashing

```bash
# Flash and monitor
espflash flash --monitor target/xtensa-esp32-espidf/release/esp32-avi-device

# Or just flash
espflash flash target/xtensa-esp32-espidf/release/esp32-avi-device
```

## Monitoring Serial Output

```bash
espflash monitor
```

## Project Structure

```
esp32-device-standalone/
├── Cargo.toml              # Project dependencies
├── build.rs                # ESP-IDF build script
├── sdkconfig.defaults      # ESP32 configuration
├── .cargo/
│   └── config.toml         # Target configuration
└── src/
    └── main.rs             # Main application code
```

## What This Does

The device will:
1. Connect to WiFi
2. Connect to the gateway at the specified IP
3. Subscribe to command and sensor topics
4. Periodically send:
   - Temperature readings
   - Humidity readings
   - Button press events
   - Status updates

## Customization

### Add Your Own Sensors

```rust
// Add to main loop in src/main.rs
let my_value = read_my_sensor();
device.update_sensor("my_sensor", SensorValue::Raw(my_value)).await?;
```

### Handle Custom Commands

```rust
// Edit DeviceHandler::on_message in src/main.rs
impl MessageHandler for DeviceHandler {
    fn on_message(&mut self, topic: &str, data: &[u8]) {
        if topic == "avi/home/device_1234/my_command" {
            // Handle your command
        }
    }
}
```

### Change Update Frequency

```rust
// Adjust delays in main loop
if counter % 50 == 0 {  // Change this number
    // Update sensor
}
```

## Troubleshooting

### Cannot connect to WiFi
- Verify SSID and password
- Check that WiFi is 2.4GHz (ESP32 doesn't support 5GHz)
- Increase timeout in code if weak signal

### Cannot reach gateway
- Ping gateway IP from another device
- Check firewall allows UDP port 8888
- Verify device and gateway on same network

### Build errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Stack overflow
Increase stack size in `sdkconfig.defaults`:
```
CONFIG_ESP_MAIN_TASK_STACK_SIZE=16000
```

## Next Steps

1. Flash this to your ESP32
2. Start the gateway on your PC: `cargo run --example gateway_with_pubsub`
3. Watch the serial monitor for connection status
4. See messages appear in the gateway terminal

Enjoy your AVI P2P embedded device! 🚀
