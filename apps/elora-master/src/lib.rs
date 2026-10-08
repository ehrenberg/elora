//! Master server for the internet server list (M7.8, E-112, E-127).
//!
//! HTTP with JSON as in `DDNet`:
//!
//! - `POST /register` with `{"port": 8303, "version": 2}` – a game server registers.
//!   The master only lists it once it reaches the server itself via a UDP info query
//!   and the protocol version matches; entries expire after [`EXPIRY`], servers
//!   re-register every [`REGISTER_INTERVAL`].
//! - `GET /servers` returns `{"servers": ["1.2.3.4:8303", …]}`. The client queries the
//!   details (name, map, ping, players) from each server itself.
//!
//! In public, the master runs behind a reverse proxy with HTTPS (E-127); then
//! `X-Forwarded-For` names the address of the game server (only with `--behind-proxy`).

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod registry;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod service;

/// Game servers re-register at this interval.
pub const REGISTER_INTERVAL: Duration = Duration::from_secs(20);
/// Without a new registration a server is dropped from the list after this time.
pub const EXPIRY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// UDP port of the game server (the master takes the IP from the connection).
    pub port: u16,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterReply {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerList {
    pub servers: Vec<String>,
}
