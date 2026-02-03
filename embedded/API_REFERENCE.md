# AVI Embedded C API - Quick Reference

## Initialization
```c
void avi_embedded_init(void);

CAviEmbedded* avi_embedded_new(
    CAviEmbeddedConfig config,      // {.device_id = 0x123...}
    uint8_t* buffer,                // 2048 bytes recommended
    size_t buffer_len,
    void* udp_user_data,
    avi_udp_send_callback_t send_fn,
    avi_udp_receive_callback_t recv_fn,
    void* msg_user_data,
    avi_message_callback_t msg_callback
);

void avi_embedded_free(CAviEmbedded* avi);
```

## Connection
```c
int32_t avi_embedded_connect(CAviEmbedded* avi);
bool avi_embedded_is_connected(const CAviEmbedded* avi);
```

## Pub/Sub
```c
// Subscribe
int32_t avi_embedded_subscribe(
    CAviEmbedded* avi,
    const char* topic,    // Not null-terminated
    size_t topic_len
);

// Publish
int32_t avi_embedded_publish(
    CAviEmbedded* avi,
    const char* topic,
    size_t topic_len,
    const uint8_t* data,
    size_t data_len       // Max 256 bytes
);

// Unsubscribe
int32_t avi_embedded_unsubscribe(
    CAviEmbedded* avi,
    const char* topic,
    size_t topic_len
);
```

## Sensors (5 Types)
```c
// Temperature (°C)
int32_t avi_embedded_update_sensor_temperature(
    CAviEmbedded* avi,
    const char* sensor_name,
    size_t sensor_name_len,
    float value
);

// Humidity (%)
int32_t avi_embedded_update_sensor_humidity(
    CAviEmbedded* avi,
    const char* sensor_name,
    size_t sensor_name_len,
    float value
);

// Battery (0-100)
int32_t avi_embedded_update_sensor_battery(
    CAviEmbedded* avi,
    const char* sensor_name,
    size_t sensor_name_len,
    uint8_t value
);

// Status (bool)
int32_t avi_embedded_update_sensor_status(
    CAviEmbedded* avi,
    const char* sensor_name,
    size_t sensor_name_len,
    bool value
);

// Raw (i32)
int32_t avi_embedded_update_sensor_raw(
    CAviEmbedded* avi,
    const char* sensor_name,
    size_t sensor_name_len,
    int32_t value
);
```

## Button Events
```c
// Press types
#define PRESS_TYPE_SINGLE  0
#define PRESS_TYPE_DOUBLE  1
#define PRESS_TYPE_LONG    2

int32_t avi_embedded_button_pressed(
    CAviEmbedded* avi,
    uint8_t button_id,
    uint8_t press_type
);
```

## Audio Streaming
```c
// Start stream
int32_t avi_embedded_start_stream(
    CAviEmbedded* avi,
    uint8_t local_stream_id,       // 1-255
    const char* target_peer_id,
    size_t target_peer_id_len,
    const char* reason,
    size_t reason_len
);

// Send data
int32_t avi_embedded_send_stream_data(
    CAviEmbedded* avi,
    uint8_t local_stream_id,
    const uint8_t* data,
    size_t data_len                // Max 512 bytes
);

// Close stream
int32_t avi_embedded_close_stream(
    CAviEmbedded* avi,
    uint8_t local_stream_id
);
```

## Message Processing
```c
int32_t avi_embedded_poll(CAviEmbedded* avi);
```

## Return Values
- `0`: Success
- `-1`: Error (null pointer, invalid parameters)
- `-2`: Queue full (retry later)

## Callbacks
```c
// Message received
typedef void (*avi_message_callback_t)(
    void* user_data,
    const char* topic,
    size_t topic_len,
    const uint8_t* data,
    size_t data_len
);

// UDP send (return 0 on success)
typedef int32_t (*avi_udp_send_callback_t)(
    void* user_data,
    const uint8_t* buf,
    size_t len
);

// UDP receive (return bytes received, 0 if none, -1 on error)
typedef int32_t (*avi_udp_receive_callback_t)(
    void* user_data,
    uint8_t* buf,
    size_t buf_len
);
```

## Common Patterns

### Initialization
```c
avi_embedded_init();

CAviEmbeddedConfig config = { .device_id = 0x123456789ABCDEF0ULL };
static uint8_t buffer[2048];

CAviEmbedded* avi = avi_embedded_new(
    config, buffer, sizeof(buffer),
    &udp_ctx, udp_send, udp_receive,
    NULL, on_message_received
);
```

### Main Loop
```c
while (1) {
    avi_embedded_poll(avi);
    
    // Send sensor updates
    avi_embedded_update_sensor_temperature(avi, "temp", 4, 22.5f);
    
    // Handle other tasks...
    vTaskDelay(pdMS_TO_TICKS(100));
}
```

### Streaming Example
```c
uint8_t stream_id = 1;

// Start
avi_embedded_start_stream(avi, stream_id, "server", 6, "voice", 5);

// Send chunks
for (int i = 0; i < num_chunks; i++) {
    avi_embedded_send_stream_data(avi, stream_id, chunk, chunk_len);
    vTaskDelay(pdMS_TO_TICKS(20));
}

// Close
avi_embedded_close_stream(avi, stream_id);
```

## Protocol Mapping

### C API → Rust Protocol

| C Function | Protocol Message |
|------------|------------------|
| `avi_embedded_connect()` | `UplinkMessage::Hello` |
| `avi_embedded_subscribe()` | `UplinkMessage::Subscribe` |
| `avi_embedded_publish()` | `UplinkMessage::Publish` |
| `avi_embedded_update_sensor_temperature()` | `UplinkMessage::SensorUpdate { data: Temperature(...) }` |
| `avi_embedded_button_pressed()` | `UplinkMessage::ButtonPress` |
| `avi_embedded_start_stream()` | `UplinkMessage::StreamStart` |
| `avi_embedded_send_stream_data()` | `UplinkMessage::StreamData` |

### Server → Device (via callback)

| Protocol Message | Callback |
|------------------|----------|
| `DownlinkMessage::Message` | `on_message_received()` |
| `DownlinkMessage::Welcome` | (Internal) |
| `DownlinkMessage::SubscribeAck` | (Internal) |

## Size Limits

| Item | Max Size | Notes |
|------|----------|-------|
| Topic name | 64 bytes | For subscribe/publish |
| Publish data | 256 bytes | Per message |
| Stream data | 512 bytes | Per chunk |
| Sensor name | 32 bytes | |
| Queue size | 16 commands | Increase `QUEUE_SIZE` if needed |

## Memory Usage

- **Instance**: ~4KB
- **Scratch buffer**: 2048 bytes (user-provided)
- **Stack**: 8KB per task (recommended)
- **Total heap**: ~20-30KB

## Thread Safety

- All functions are **non-blocking**
- Commands are queued and processed asynchronously
- Safe to call from multiple FreeRTOS tasks
- Callbacks are called from async executor context

## Error Handling

```c
int ret = avi_embedded_publish(avi, topic, topic_len, data, data_len);
if (ret == 0) {
    // Success - command queued
} else if (ret == -1) {
    // Error - bad parameters
} else if (ret == -2) {
    // Queue full - retry later
}
```
