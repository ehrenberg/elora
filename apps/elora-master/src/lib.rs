//! Master-Server für die Internet-Serverliste (M7.8, E-112, E-127).
//!
//! HTTP mit JSON wie bei `DDNet`:
//!
//! - `POST /register` mit `{"port": 8303, "version": 2}` – ein Spielserver meldet sich
//!   an. Der Master nimmt ihn erst auf, wenn er ihn selbst per UDP-Info-Abfrage
//!   erreicht und die Protokollversion passt; Einträge laufen nach [`EXPIRY`] ab,
//!   Server melden sich alle [`REGISTER_INTERVAL`] neu.
//! - `GET /servers` liefert `{"servers": ["1.2.3.4:8303", …]}`. Details (Name, Karte,
//!   Ping, Spieler) fragt der Client bei jedem Server selbst ab.
//!
//! Öffentlich läuft der Master hinter einem Reverse-Proxy mit HTTPS (E-127); dann
//! nennt `X-Forwarded-For` die Adresse des Spielservers (nur mit `--behind-proxy`).

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod registry;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod service;

/// So oft melden sich Spielserver neu an.
pub const REGISTER_INTERVAL: Duration = Duration::from_secs(20);
/// Ohne neue Anmeldung fliegt ein Server nach dieser Zeit aus der Liste.
pub const EXPIRY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// UDP-Port des Spielservers (die IP nimmt der Master aus der Verbindung).
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
