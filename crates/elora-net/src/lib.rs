//! UDP transport of Elora (E-012): handshake with token, Noise encryption
//! (E-061, E-062), reliable and unreliable messages, network simulator.
//!
//! Knows nothing about game content – bytes in, bytes out.

pub mod endpoint;
pub mod info;
pub mod session;
pub mod socket;

pub use endpoint::{
    ClientEndpoint, ClientEvent, DisconnectReason, Keypair, ServerEndpoint, ServerEvent, hex,
};
pub use info::{INFO_TIMEOUT, InfoProbe, InfoReply};
pub use session::{Delivery, Session, SessionError, Stats};
pub use socket::{Conditioner, Conditions, MemNetwork, MemSocket, Socket, UdpSocket};
