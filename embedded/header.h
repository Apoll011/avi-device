#ifndef AVI_EMBEDDED_H
#define AVI_EMBEDDED_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle to the AVI embedded instance
typedef struct CAviEmbedded CAviEmbedded;

// Configuration structure
typedef struct {
    uint64_t device_id;
} CAviEmbeddedConfig;

// Callback function types
typedef void (*CMessageCallback)(void* user_data, const char* topic, size_t topic_len, 
                                  const uint8_t* data, size_t data_len);
typedef int32_t (*CUdpSendCallback)(void* user_data, const uint8_t* buf, size_t len);
typedef int32_t (*CUdpReceiveCallback)(void* user_data, uint8_t* buf, size_t buf_len);

// Press types for buttons
typedef enum {
    PRESS_TYPE_SHORT = 0,
    PRESS_TYPE_LONG = 1,
    PRESS_TYPE_DOUBLE = 2
} PressType;

/**
 * Initialize the AVI embedded system (call once at startup before creating instances)
 * This initializes the global command queue for async operations
 */
void avi_embedded_init(void);

/**
 * Create a new AVI embedded instance
 * 
 * @param config Configuration for the device
 * @param buffer Scratch buffer for serialization (recommend 2048+ bytes)
 * @param buffer_len Length of the scratch buffer
 * @param udp_user_data User data passed to UDP callbacks
 * @param udp_send_fn Callback for sending UDP packets
 * @param udp_recv_fn Callback for receiving UDP packets
 * @param msg_user_data User data passed to message callbacks
 * @param msg_callback Callback for received pub/sub messages
 * @return Pointer to the AVI instance, or NULL on failure
 */
CAviEmbedded* avi_embedded_new(
    CAviEmbeddedConfig config,
    uint8_t* buffer,
    size_t buffer_len,
    void* udp_user_data,
    CUdpSendCallback udp_send_fn,
    CUdpReceiveCallback udp_recv_fn,
    void* msg_user_data,
    CMessageCallback msg_callback
);

/**
 * Free an AVI embedded instance
 * 
 * @param avi The instance to free
 */
void avi_embedded_free(CAviEmbedded* avi);

/**
 * Connect to the AVI server (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @return 0 on success, -1 on invalid instance, -2 if queue is full
 */
int32_t avi_embedded_connect(CAviEmbedded* avi);

/**
 * Check if connected to the server
 * 
 * @param avi The AVI instance
 * @return true if connected, false otherwise
 */
bool avi_embedded_is_connected(const CAviEmbedded* avi);

/**
 * Subscribe to a topic (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param topic The topic string (max 64 bytes)
 * @param topic_len Length of the topic string
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_subscribe(CAviEmbedded* avi, const char* topic, size_t topic_len);

/**
 * Unsubscribe from a topic (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param topic The topic string (max 64 bytes)
 * @param topic_len Length of the topic string
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_unsubscribe(CAviEmbedded* avi, const char* topic, size_t topic_len);

/**
 * Publish data to a topic (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param topic The topic string (max 64 bytes)
 * @param topic_len Length of the topic string
 * @param data The data to publish (max 256 bytes)
 * @param data_len Length of the data
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_publish(CAviEmbedded* avi, const char* topic, size_t topic_len,
                               const uint8_t* data, size_t data_len);

/**
 * Start an audio stream (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param stream_id Local stream identifier
 * @param target_peer Target peer ID string (max 64 bytes)
 * @param target_peer_len Length of target peer string
 * @param reason Reason for the stream (max 64 bytes)
 * @param reason_len Length of reason string
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_start_stream(CAviEmbedded* avi, uint8_t stream_id,
                                    const char* target_peer, size_t target_peer_len,
                                    const char* reason, size_t reason_len);

/**
 * Send audio data on a stream (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param stream_id The stream identifier
 * @param pcm_data PCM audio data (max 512 bytes)
 * @param pcm_len Length of PCM data
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_send_audio(CAviEmbedded* avi, uint8_t stream_id,
                                 const uint8_t* pcm_data, size_t pcm_len);

/**
 * Close an audio stream (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param stream_id The stream identifier
 * @return 0 on success, -1 on invalid instance, -2 if queue is full
 */
int32_t avi_embedded_close_stream(CAviEmbedded* avi, uint8_t stream_id);

/**
 * Send a button press event (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param button_id The button identifier
 * @param press_type Type of press (0=short, 1=long, 2=double)
 * @return 0 on success, -1 on invalid instance, -2 if queue is full
 */
int32_t avi_embedded_button_pressed(CAviEmbedded* avi, uint8_t button_id, uint8_t press_type);

/**
 * Update a sensor with a float value (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param name Sensor name (max 32 bytes)
 * @param name_len Length of sensor name
 * @param value Float value
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_update_sensor_float(CAviEmbedded* avi, const char* name, 
                                          size_t name_len, float value);

/**
 * Update a sensor with an integer value (non-blocking - queues the command)
 * 
 * @param avi The AVI instance
 * @param name Sensor name (max 32 bytes)
 * @param name_len Length of sensor name
 * @param value Integer value
 * @return 0 on success, -1 on invalid parameters, -2 if queue is full
 */
int32_t avi_embedded_update_sensor_int(CAviEmbedded* avi, const char* name, 
                                        size_t name_len, int32_t value);

/**
 * Poll for incoming messages (non-blocking - queues the command)
 * Call this regularly in your main loop to process incoming messages
 * 
 * @param avi The AVI instance
 * @return 0 on success, -1 on invalid instance, -2 if queue is full
 */
int32_t avi_embedded_poll(CAviEmbedded* avi);

#ifdef __cplusplus
}
#endif

#endif // AVI_EMBEDDED_H
