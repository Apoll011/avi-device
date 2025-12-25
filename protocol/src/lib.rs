#![no_std]
use serde::{Serialize, Deserialize};

pub const MAX_PACKET_SIZE: usize = 1024;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PressType {
    Single,
    Double,
    Long,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorValue {
    Temperature(f32),
    Humidity(f32),
    Battery(u8),
    Status(bool), // On/Off
    Raw(i32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UplinkMessage<'a> {
    Hello { device_id: u64 },

    StreamStart { local_stream_id: u8, target_peer_id: &'a str, reason: &'a str },
    StreamData { local_stream_id: u8, #[serde(with = "serde_bytes")] data: &'a [u8] },
    StreamClose { local_stream_id: u8 },

    ButtonPress {
        button_id: u8,
        press_type: PressType
    },

    SensorUpdate {
        sensor_name: &'a str, // e.g., "temp_kitchen"
        data: SensorValue
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DownlinkMessage {
    Welcome,
    Error { reason: u8 },
}