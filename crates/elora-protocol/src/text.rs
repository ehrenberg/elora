//! Translatable server messages (M8.1, O-48, E-164).
//!
//! The server sends codes with values; the client shows them in its language. The German
//! representation ([`std::fmt::Display`]) is the server's language for log and console and
//! the fallback for unknown codes.
//!
//! Disconnect reasons pass through `elora-net` as text; for them there are codes ([`reason`])
//! that start with `#`. The client shows other texts unchanged.

use std::fmt;

use elora_game::Mode;
use elora_sim::Team;

use crate::codec::{DecodeError, DecodeResult, Reader, Writer};
use crate::msg::{team_code, team_from};

const MAX_TEXT: usize = 256;
const MAX_NAME: usize = 128;

/// Winner of a round or a match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WinnerName {
    Player(String),
    Team(Team),
    Nobody,
}

/// Subject of a vote, as it is displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteSubject {
    Map(String),
    Mode { mode: Mode, instagib: bool },
    Kick(String),
    Spectate(String),
}

/// A message from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Free text (e.g. console); not translated.
    Text(String),
    Joined {
        name: String,
    },
    Left {
        name: String,
    },
    /// Player changes team on their own (also to the spectators or into free play).
    TeamJoined {
        name: String,
        team: Team,
    },
    /// Balancing by the server.
    TeamBalanced {
        name: String,
        team: Team,
    },
    MatchStarted {
        mode: String,
    },
    RoundWon(WinnerName),
    RoundDraw,
    MatchWon(WinnerName),
    SuddenDeath,
    MapChanged {
        map: String,
    },
    ModeChanged {
        mode: String,
    },
    VoteStarted {
        who: String,
        subject: VoteSubject,
    },
    VoteFailed(VoteSubject),
    VotePassed(VoteSubject),
    VoteCancelled,
    VotesDisabled,
    VoteRunning,
    UnknownMap {
        map: String,
    },
    InvalidPlayer,
    MapChangeFailed {
        map: String,
    },
}

/// Disconnect reasons as codes (pass through `elora-net` as text).
pub mod reason {
    pub const BANNED: &str = "#banned";
    pub const INVALID_MESSAGE: &str = "#invalid-message";
    pub const WRONG_VERSION: &str = "#wrong-version";
    pub const LEFT: &str = "#left";
    pub const SHUTDOWN: &str = "#shutdown";
    pub const KICKED_BY_VOTE: &str = "#kicked-by-vote";
    pub const KICKED: &str = "#kicked";
    pub const SERVER_FULL: &str = "#server-full";

    /// All codes (for tests of the translations).
    pub const ALL: [&str; 8] = [
        BANNED,
        INVALID_MESSAGE,
        WRONG_VERSION,
        LEFT,
        SHUTDOWN,
        KICKED_BY_VOTE,
        KICKED,
        SERVER_FULL,
    ];

    /// Language key of a code (`reason.<code>`), otherwise `None` (free text).
    pub fn key(text: &str) -> Option<String> {
        let code = text.strip_prefix('#')?;
        ALL.contains(&text).then(|| format!("reason.{code}"))
    }

    /// German text for log and old clients.
    pub fn german(text: &str) -> &str {
        match text {
            BANNED => "Du bist vorübergehend gesperrt",
            INVALID_MESSAGE => "Ungültige Nachricht",
            WRONG_VERSION => "Falsche Spielversion",
            LEFT => "Verlassen",
            SHUTDOWN => "Server wird beendet",
            KICKED_BY_VOTE => "Per Abstimmung gekickt",
            KICKED => "Vom Server getrennt",
            SERVER_FULL => "Server ist voll",
            other => other,
        }
    }
}

fn text(r: &mut Reader<'_>, max: usize) -> DecodeResult<String> {
    Ok(r.str(max)?.to_owned())
}

fn put_team(w: &mut Writer, t: Team) {
    w.u8(team_code(t));
}

fn put_winner(w: &mut Writer, n: &WinnerName) {
    match n {
        WinnerName::Player(p) => {
            w.u8(0);
            w.str(p);
        }
        WinnerName::Team(t) => {
            w.u8(1);
            put_team(w, *t);
        }
        WinnerName::Nobody => w.u8(2),
    }
}

fn get_winner(r: &mut Reader<'_>) -> DecodeResult<WinnerName> {
    Ok(match r.u8()? {
        0 => WinnerName::Player(r.str(MAX_NAME)?.to_owned()),
        1 => WinnerName::Team(team_from(r.u8()?)?),
        2 => WinnerName::Nobody,
        _ => return Err(DecodeError::Invalid("Gewinner")),
    })
}

impl VoteSubject {
    pub(crate) fn put(&self, w: &mut Writer) {
        match self {
            Self::Map(m) => {
                w.u8(0);
                w.str(m);
            }
            Self::Mode { mode, instagib } => {
                w.u8(1);
                w.u8(mode.index());
                w.bool(*instagib);
            }
            Self::Kick(n) => {
                w.u8(2);
                w.str(n);
            }
            Self::Spectate(n) => {
                w.u8(3);
                w.str(n);
            }
        }
    }

    pub(crate) fn get(r: &mut Reader<'_>) -> DecodeResult<Self> {
        Ok(match r.u8()? {
            0 => Self::Map(r.str(MAX_NAME)?.to_owned()),
            1 => Self::Mode {
                mode: Mode::from_index(r.u8()?).ok_or(DecodeError::Invalid("Modus"))?,
                instagib: r.bool()?,
            },
            2 => Self::Kick(r.str(MAX_NAME)?.to_owned()),
            3 => Self::Spectate(r.str(MAX_NAME)?.to_owned()),
            _ => return Err(DecodeError::Invalid("Abstimmung")),
        })
    }

    /// Mode as a short name (e.g. `iCTF`).
    pub fn mode_label(mode: Mode, instagib: bool) -> String {
        format!("{}{}", if instagib { "i" } else { "" }, mode.name())
    }
}

impl Message {
    pub(crate) fn put(&self, w: &mut Writer) {
        let named = |w: &mut Writer, code: u8, s: &str| {
            w.u8(code);
            w.str(s);
        };
        match self {
            Self::Text(t) => named(w, 0, t),
            Self::Joined { name } => named(w, 1, name),
            Self::Left { name } => named(w, 2, name),
            Self::TeamJoined { name, team } => {
                named(w, 3, name);
                put_team(w, *team);
            }
            Self::TeamBalanced { name, team } => {
                named(w, 4, name);
                put_team(w, *team);
            }
            Self::MatchStarted { mode } => named(w, 5, mode),
            Self::RoundWon(n) => {
                w.u8(6);
                put_winner(w, n);
            }
            Self::RoundDraw => w.u8(7),
            Self::MatchWon(n) => {
                w.u8(8);
                put_winner(w, n);
            }
            Self::SuddenDeath => w.u8(9),
            Self::MapChanged { map } => named(w, 10, map),
            Self::ModeChanged { mode } => named(w, 11, mode),
            Self::VoteStarted { who, subject } => {
                named(w, 12, who);
                subject.put(w);
            }
            Self::VoteFailed(s) => {
                w.u8(13);
                s.put(w);
            }
            Self::VotePassed(s) => {
                w.u8(14);
                s.put(w);
            }
            Self::VoteCancelled => w.u8(15),
            Self::VotesDisabled => w.u8(16),
            Self::VoteRunning => w.u8(17),
            Self::UnknownMap { map } => named(w, 18, map),
            Self::InvalidPlayer => w.u8(19),
            Self::MapChangeFailed { map } => named(w, 20, map),
        }
    }

    pub(crate) fn get(r: &mut Reader<'_>) -> DecodeResult<Self> {
        let code = r.u8()?;
        Ok(match code {
            0 => Self::Text(text(r, MAX_TEXT)?),
            1 => Self::Joined {
                name: text(r, MAX_NAME)?,
            },
            2 => Self::Left {
                name: text(r, MAX_NAME)?,
            },
            3 => Self::TeamJoined {
                name: text(r, MAX_NAME)?,
                team: team_from(r.u8()?)?,
            },
            4 => Self::TeamBalanced {
                name: text(r, MAX_NAME)?,
                team: team_from(r.u8()?)?,
            },
            5 => Self::MatchStarted {
                mode: text(r, MAX_NAME)?,
            },
            6 => Self::RoundWon(get_winner(r)?),
            7 => Self::RoundDraw,
            8 => Self::MatchWon(get_winner(r)?),
            9 => Self::SuddenDeath,
            10 => Self::MapChanged {
                map: text(r, MAX_NAME)?,
            },
            11 => Self::ModeChanged {
                mode: text(r, MAX_NAME)?,
            },
            12 => Self::VoteStarted {
                who: text(r, MAX_NAME)?,
                subject: VoteSubject::get(r)?,
            },
            13 => Self::VoteFailed(VoteSubject::get(r)?),
            14 => Self::VotePassed(VoteSubject::get(r)?),
            15 => Self::VoteCancelled,
            16 => Self::VotesDisabled,
            17 => Self::VoteRunning,
            18 => Self::UnknownMap {
                map: text(r, MAX_NAME)?,
            },
            19 => Self::InvalidPlayer,
            20 => Self::MapChangeFailed {
                map: text(r, MAX_NAME)?,
            },
            _ => return Err(DecodeError::Invalid("Meldung")),
        })
    }
}

fn team_de(t: Team) -> &'static str {
    match t {
        Team::Red => "Rot",
        Team::Blue => "Blau",
        Team::Spectator => "die Zuschauer",
        Team::None => "das Spiel",
    }
}

impl fmt::Display for WinnerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Player(p) => f.write_str(p),
            Self::Team(Team::Red) => f.write_str("Team Rot"),
            Self::Team(Team::Blue) => f.write_str("Team Blau"),
            Self::Team(_) | Self::Nobody => f.write_str("niemand"),
        }
    }
}

impl fmt::Display for VoteSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Map(m) => write!(f, "Karte wechseln: {m}"),
            Self::Mode { mode, instagib } => {
                write!(f, "Modus wechseln: {}", Self::mode_label(*mode, *instagib))
            }
            Self::Kick(n) => write!(f, "{n} kicken"),
            Self::Spectate(n) => write!(f, "{n} zu den Zuschauern"),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t) => f.write_str(t),
            Self::Joined { name } => write!(f, "{name} ist beigetreten"),
            Self::Left { name } => write!(f, "{name} hat das Spiel verlassen"),
            Self::TeamJoined { name, team } => write!(f, "{name} wechselt zu {}", team_de(*team)),
            Self::TeamBalanced { name, team } => {
                write!(f, "{name} wechselt zum Ausgleich zu {}", team_de(*team))
            }
            Self::MatchStarted { mode } => write!(f, "{mode} – Match beginnt"),
            Self::RoundWon(w) => write!(f, "{w} gewinnt die Runde"),
            Self::RoundDraw => f.write_str("Unentschieden"),
            Self::MatchWon(w) => write!(f, "{w} gewinnt das Match!"),
            Self::SuddenDeath => f.write_str("Gleichstand – Sudden Death!"),
            Self::MapChanged { map } => write!(f, "Karte: {map}"),
            Self::ModeChanged { mode } => write!(f, "Modus: {mode}"),
            Self::VoteStarted { who, subject } => {
                write!(f, "{who} startet eine Abstimmung: {subject}")
            }
            Self::VoteFailed(s) => write!(f, "Abstimmung abgelehnt: {s}"),
            Self::VotePassed(s) => write!(f, "Abstimmung angenommen: {s}"),
            Self::VoteCancelled => f.write_str("Abstimmung abgebrochen"),
            Self::VotesDisabled => f.write_str("Abstimmungen sind auf diesem Server abgeschaltet"),
            Self::VoteRunning => f.write_str("Es läuft bereits eine Abstimmung"),
            Self::UnknownMap { map } => write!(f, "Karte `{map}` unbekannt"),
            Self::InvalidPlayer => f.write_str("Ungültiger Spieler"),
            Self::MapChangeFailed { map } => write!(f, "Kartenwechsel fehlgeschlagen: {map}"),
        }
    }
}

#[cfg(test)]
pub(crate) fn samples() -> Vec<Message> {
    let s = || VoteSubject::Mode {
        mode: Mode::Ctf,
        instagib: true,
    };
    vec![
        Message::Text("Hallo".into()),
        Message::Joined { name: "A".into() },
        Message::Left { name: "A".into() },
        Message::TeamJoined {
            name: "A".into(),
            team: Team::Spectator,
        },
        Message::TeamBalanced {
            name: "A".into(),
            team: Team::Blue,
        },
        Message::MatchStarted { mode: "CTF".into() },
        Message::RoundWon(WinnerName::Player("A".into())),
        Message::RoundDraw,
        Message::MatchWon(WinnerName::Team(Team::Red)),
        Message::SuddenDeath,
        Message::MapChanged {
            map: "dm-wiese".into(),
        },
        Message::ModeChanged {
            mode: "iTDM".into(),
        },
        Message::VoteStarted {
            who: "A".into(),
            subject: VoteSubject::Kick("B".into()),
        },
        Message::VoteFailed(s()),
        Message::VotePassed(VoteSubject::Map("ctf-wald".into())),
        Message::VoteCancelled,
        Message::VotesDisabled,
        Message::VoteRunning,
        Message::UnknownMap { map: "x".into() },
        Message::InvalidPlayer,
        Message::MapChangeFailed { map: "x".into() },
        Message::RoundWon(WinnerName::Nobody),
        Message::VoteStarted {
            who: "A".into(),
            subject: VoteSubject::Spectate("B".into()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_roundtrip() {
        for m in samples() {
            let mut w = Writer::new();
            m.put(&mut w);
            let data = w.into_bytes();
            let mut r = Reader::new(&data);
            assert_eq!(Message::get(&mut r).unwrap(), m);
            assert!(!m.to_string().is_empty());
        }
    }

    #[test]
    fn reason_codes() {
        assert_eq!(
            reason::key(reason::BANNED).as_deref(),
            Some("reason.banned")
        );
        assert_eq!(reason::key("irgendwas"), None);
        assert_eq!(reason::key("#unbekannt"), None);
        assert_eq!(reason::german(reason::SERVER_FULL), "Server ist voll");
        assert_eq!(reason::german("frei"), "frei");
    }
}
