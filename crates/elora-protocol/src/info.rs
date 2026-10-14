//! Server info for the server browser (M7.6): answer to the connectionless
//! info query of `elora-net`.

use elora_sim::Team;

use crate::PROTOCOL_VERSION;
use crate::codec::{DecodeError, DecodeResult, Reader, Writer};
use crate::msg::{team_code, team_from};

/// At most this many players are listed (the answer must fit into one datagram).
pub const MAX_LISTED: usize = 32;
const MAX_NAME: usize = 64;
const MAX_TEXT: usize = 128;

/// One player in the server info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoPlayer {
    /// Player name.
    pub name: String,
    /// Current score.
    pub score: i32,
    /// Team (or spectator).
    pub team: Team,
    /// Whether the player is a server-side dummy.
    pub dummy: bool,
}

/// Reply to an info query (server browser).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInfo {
    /// Protocol version of the server; connecting only works if they are equal.
    pub version: u32,
    /// Server name.
    pub name: String,
    /// Name of the current map.
    pub map: String,
    /// Mode for display, e.g. `CTF` or `iDM`.
    pub mode: String,
    /// Connected humans (without dummies, spectators included).
    pub clients: u32,
    /// Maximum number of clients.
    pub max_clients: u32,
    /// Player list (at most [`MAX_LISTED`]).
    pub players: Vec<InfoPlayer>,
}

impl ServerInfo {
    /// Whether the client can connect (same protocol version).
    pub fn compatible(&self) -> bool {
        self.version == PROTOCOL_VERSION
    }

    /// Encodes the info for the network.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.uvar(u64::from(self.version));
        w.str(&self.name);
        w.str(&self.map);
        w.str(&self.mode);
        w.uvar(u64::from(self.clients));
        w.uvar(u64::from(self.max_clients));
        let listed = &self.players[..self.players.len().min(MAX_LISTED)];
        w.uvar(listed.len() as u64);
        for p in listed {
            w.str(&p.name);
            w.ivar(i64::from(p.score));
            w.u8(team_code(p.team));
            w.bool(p.dummy);
        }
        w.into_bytes()
    }

    /// # Errors
    /// On faulty data.
    pub fn decode(data: &[u8]) -> DecodeResult<Self> {
        let mut r = Reader::new(data);
        let version = r.uint("version")?;
        let name = r.str(MAX_TEXT)?.to_owned();
        let map = r.str(MAX_TEXT)?.to_owned();
        let mode = r.str(MAX_NAME)?.to_owned();
        let clients = r.uint("players")?;
        let max_clients = r.uint("players")?;
        let n: usize = r.uint("count")?;
        if n > MAX_LISTED {
            return Err(DecodeError::Invalid("count"));
        }
        let mut players = Vec::with_capacity(n);
        for _ in 0..n {
            let name = r.str(MAX_NAME)?.to_owned();
            let score = i32::try_from(r.ivar()?).map_err(|_| DecodeError::Overflow)?;
            let team = team_from(r.u8()?)?;
            let dummy = r.bool()?;
            players.push(InfoPlayer {
                name,
                score,
                team,
                dummy,
            });
        }
        r.finish()?;
        Ok(Self {
            version,
            name,
            map,
            mode,
            clients,
            max_clients,
            players,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_limit() {
        let player = |i: i32| InfoPlayer {
            name: format!("Player {i}"),
            score: i - 3,
            team: if i % 2 == 0 { Team::Red } else { Team::Blue },
            dummy: i == 1,
        };
        let info = ServerInfo {
            version: PROTOCOL_VERSION,
            name: "Eloras Wiese".into(),
            map: "ctf-test".into(),
            mode: "iCTF".into(),
            clients: 40,
            max_clients: 64,
            players: (0..40).map(player).collect(),
        };
        let data = info.encode();
        assert!(data.len() < 1400, "fits into one datagram: {}", data.len());
        let back = ServerInfo::decode(&data).unwrap();
        assert_eq!(back.players.len(), MAX_LISTED);
        assert_eq!(back.players[..], info.players[..MAX_LISTED]);
        assert_eq!(back.clients, 40);
        assert!(back.compatible());
        assert!(ServerInfo::decode(&data[..data.len() - 1]).is_err());
    }
}
