//! Netzwerkprotokoll von Elora (E-008): Nachrichten, Serialisierung, Snapshots.
//!
//! Kennt keine Sockets – Transport, Verschlüsselung und Zuverlässigkeit liegen in
//! `elora-net`.

pub mod codec;
pub mod msg;
pub mod snapshot;

pub use msg::{ClientMsg, ServerMsg};
pub use snapshot::Snapshot;

/// Version des Spielprotokolls; Client und Server müssen übereinstimmen.
pub const PROTOCOL_VERSION: u32 = 1;
