//! UDP-Transport von Elora (E-012): Handshake mit Token, Noise-Verschlüsselung
//! (E-061, E-062), zuverlässige und unzuverlässige Nachrichten, Netzwerk-Simulator.
//!
//! Kennt keine Spielinhalte – Bytes rein, Bytes raus.

pub mod endpoint;
pub mod session;
pub mod socket;

pub use endpoint::{
    ClientEndpoint, ClientEvent, DisconnectReason, Keypair, ServerEndpoint, ServerEvent, hex,
};
pub use session::{Delivery, Session, SessionError, Stats};
pub use socket::{Conditioner, Conditions, MemNetwork, MemSocket, Socket, UdpSocket};
