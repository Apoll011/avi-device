//! # AVI P2P
//!
//! A production-ready, opinionated abstraction over libp2p for the AVI Core platform.
//!
//! ## Features
//! - Pub/Sub messaging
//! - Streaming (logical streams over request-response)
//! - Kademlia Mesh Networking
//! - Zero libp2p type exposure

mod behaviour;
pub mod bridge;
mod command;
pub mod config;
mod error;
pub mod events;
mod node;
mod protocols;
mod runtime;

pub use bridge::{BridgeConfig, EmbeddedBridge};
pub use config::AviP2pConfig;
pub use error::{AviP2pError, StreamCloseReason};
pub use events::{AviEvent, PeerId};
pub use node::{AviP2p, AviP2pHandle};
pub use protocols::context::{delete_nested_value, set_nested_value};
pub use protocols::context::{AviContext, VectorClock};
pub use protocols::stream::{
    generate_stream_id, StreamDirection, StreamId, StreamState, StreamStatus,
};
