//! Network protocol of Elora (E-008): messages, serialization, snapshots.
//!
//! Knows no sockets – transport, encryption and reliability live in
//! `elora-net`.

pub mod codec;
pub mod huffman;
mod huffman_table;
pub mod info;
pub mod msg;
pub mod snapshot;
pub mod text;

pub use info::{InfoPlayer, ServerInfo};
pub use msg::{ClientMsg, MAP_CHUNK, MAX_MAP, MapChecksum, ServerMsg, Skin, VoteInfo, VoteKind};
pub use snapshot::{GameView, Snapshot};
pub use text::{Message, VoteSubject, WinnerName, reason};

/// Version of the game protocol; client and server must match.
pub const PROTOCOL_VERSION: u32 = 7;
