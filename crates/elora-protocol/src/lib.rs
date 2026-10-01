//! Netzwerkprotokoll von Elora (E-008): Nachrichten, Serialisierung, Snapshots.
//!
//! Kennt keine Sockets – Transport, Verschlüsselung und Zuverlässigkeit liegen in
//! `elora-net`.

pub mod codec;
pub mod huffman;
mod huffman_table;
pub mod info;
pub mod msg;
pub mod snapshot;

pub use info::{InfoPlayer, ServerInfo};
pub use msg::{ClientMsg, MAP_CHUNK, MAX_MAP, MapChecksum, ServerMsg, Skin, VoteInfo, VoteKind};
pub use snapshot::{GameView, Snapshot};

/// Version des Spielprotokolls; Client und Server müssen übereinstimmen.
pub const PROTOCOL_VERSION: u32 = 5;
