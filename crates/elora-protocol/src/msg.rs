//! Nachrichten zwischen Client und Server.
//!
//! Zuverlässig (über den zuverlässigen Kanal von `elora-net`): `Join`, `Welcome`,
//! `Tuning`, `Kick`, `Leave`. Unzuverlässig: `Input`, `Snapshot`, `InputTiming`.

// Wire-Format: Umwandlungen zwischen den Feld-Ganzzahlen und den Simulationstypen.
// Wertebereiche werden beim Dekodieren geprüft (`validate`, `Reader::int`).
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use elora_sim::{DeathCause, Event, PickupKind, PlayerInput, Tuning, Vec2, Weapon};

use crate::PROTOCOL_VERSION;
use crate::codec::{DecodeError, DecodeResult, Reader, Writer};
use crate::snapshot::Snapshot;

/// Maximale Länge von Namen und Texten.
const MAX_NAME: usize = 32;
const MAX_TEXT: usize = 256;
/// Maximale Größe einer übertragenen Karte (Textformat).
const MAX_MAP: usize = 4 * 1024 * 1024;
/// Eingaben pro Paket (Redundanz gegen Verlust).
pub const MAX_INPUTS: usize = 8;
const MAX_EVENTS: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientMsg {
    Join {
        version: u32,
        name: String,
    },
    /// Eingaben mit ihrem Ziel-Tick, neueste zuletzt. `ack` = Tick des zuletzt
    /// vollständig empfangenen Snapshots (Basis für das nächste Delta).
    Input {
        ack: Option<u64>,
        inputs: Vec<(u64, PlayerInput)>,
    },
    Leave,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMsg {
    Welcome {
        slot: u32,
        tick: u64,
        map_name: String,
        map_source: String,
        tuning: Tuning,
        high_bandwidth: bool,
    },
    /// Snapshot als Delta gegen `base` (dessen Tick der Client bestätigt hat).
    Snapshot {
        tick: u64,
        base: Option<u64>,
        checksum: u32,
        delta: Vec<u8>,
        events: Vec<Event>,
    },
    /// Zeit (ms), die die Eingabe für `tick` vor ihrer Verarbeitung ankam (negativ = zu spät).
    InputTiming {
        tick: u64,
        time_left_ms: i32,
    },
    Tuning(Tuning),
    Kick {
        reason: String,
    },
}

fn put_input(w: &mut Writer, i: &PlayerInput) {
    w.ivar(i64::from(i.direction));
    w.ivar(i64::from(i.target_x));
    w.ivar(i64::from(i.target_y));
    w.u8(u8::from(i.jump) | u8::from(i.hook) << 1);
    w.u8(i.fire);
    w.u8(i.wanted_weapon);
    w.u8(i.next_weapon);
    w.u8(i.prev_weapon);
}

fn get_input(r: &mut Reader<'_>) -> DecodeResult<PlayerInput> {
    let direction: i8 = r.int("Richtung")?;
    let target_x = r.int("Ziel")?;
    let target_y = r.int("Ziel")?;
    let flags = r.u8()?;
    let fire = r.u8()?;
    let wanted_weapon = r.u8()?;
    let next_weapon = r.u8()?;
    let prev_weapon = r.u8()?;
    if !(-1..=1).contains(&direction) || flags > 3 || wanted_weapon > 3 {
        return Err(DecodeError::Invalid("Eingabe"));
    }
    Ok(PlayerInput {
        direction,
        target_x,
        target_y,
        jump: flags & 1 != 0,
        hook: flags & 2 != 0,
        fire,
        wanted_weapon,
        next_weapon,
        prev_weapon,
    })
}

fn put_tuning(w: &mut Writer, t: &Tuning) {
    w.str(&toml::to_string(t).unwrap_or_default());
}

fn get_tuning(r: &mut Reader<'_>) -> DecodeResult<Tuning> {
    toml::from_str(r.str(64 * 1024)?).map_err(|_| DecodeError::Invalid("Tuning"))
}

fn put_vec(w: &mut Writer, v: Vec2) {
    w.ivar(v.x.round() as i64);
    w.ivar(v.y.round() as i64);
}

fn get_vec(r: &mut Reader<'_>) -> DecodeResult<Vec2> {
    Ok(Vec2::new(r.ivar()? as f32, r.ivar()? as f32))
}

fn put_opt(w: &mut Writer, v: Option<usize>) {
    w.uvar(v.map_or(0, |v| v as u64 + 1));
}

fn get_opt(r: &mut Reader<'_>) -> DecodeResult<Option<usize>> {
    let v: usize = r.uint("Index")?;
    Ok(v.checked_sub(1))
}

fn put_weapon(w: &mut Writer, x: Weapon) {
    w.u8(x.index() as u8);
}

fn get_weapon(r: &mut Reader<'_>) -> DecodeResult<Weapon> {
    Weapon::ALL
        .get(usize::from(r.u8()?))
        .copied()
        .ok_or(DecodeError::Invalid("Waffe"))
}

fn put_pickup(w: &mut Writer, k: PickupKind) {
    match k {
        PickupKind::Health => w.u8(0),
        PickupKind::Armor => w.u8(1),
        PickupKind::Weapon(x) => {
            w.u8(2);
            put_weapon(w, x);
        }
    }
}

fn get_pickup(r: &mut Reader<'_>) -> DecodeResult<PickupKind> {
    Ok(match r.u8()? {
        0 => PickupKind::Health,
        1 => PickupKind::Armor,
        2 => PickupKind::Weapon(get_weapon(r)?),
        _ => return Err(DecodeError::Invalid("Pickup")),
    })
}

/// Ereignisse (für Effekte) – Positionen gerundet auf ganze Einheiten.
fn put_event(w: &mut Writer, e: &Event) {
    match *e {
        Event::Fire {
            player,
            weapon,
            pos,
        } => {
            w.u8(0);
            w.uvar(player as u64);
            put_weapon(w, weapon);
            put_vec(w, pos);
        }
        Event::NoAmmo { player } => {
            w.u8(1);
            w.uvar(player as u64);
        }
        Event::WeaponSwitch { player, weapon } => {
            w.u8(2);
            w.uvar(player as u64);
            put_weapon(w, weapon);
        }
        Event::HammerHit { pos } => {
            w.u8(3);
            put_vec(w, pos);
        }
        Event::LaserBounce { pos } => {
            w.u8(4);
            put_vec(w, pos);
        }
        Event::Explosion { pos } => {
            w.u8(5);
            put_vec(w, pos);
        }
        Event::Damage {
            player,
            from,
            health,
            armor,
        } => {
            w.u8(6);
            w.uvar(player as u64);
            put_opt(w, from);
            w.ivar(i64::from(health));
            w.ivar(i64::from(armor));
        }
        Event::Death {
            player,
            killer,
            cause,
            pos,
        } => {
            w.u8(7);
            w.uvar(player as u64);
            put_opt(w, killer);
            match cause {
                DeathCause::World => w.u8(0xff),
                DeathCause::Weapon(x) => put_weapon(w, x),
            }
            put_vec(w, pos);
        }
        Event::Spawn { player, pos } => {
            w.u8(8);
            w.uvar(player as u64);
            put_vec(w, pos);
        }
        Event::Pickup { player, kind, pos } => {
            w.u8(9);
            w.uvar(player as u64);
            put_pickup(w, kind);
            put_vec(w, pos);
        }
        Event::PickupRespawn { kind, pos } => {
            w.u8(10);
            put_pickup(w, kind);
            put_vec(w, pos);
        }
    }
}

fn get_event(r: &mut Reader<'_>) -> DecodeResult<Event> {
    let player = |r: &mut Reader<'_>| r.uint::<usize>("Spieler");
    Ok(match r.u8()? {
        0 => Event::Fire {
            player: player(r)?,
            weapon: get_weapon(r)?,
            pos: get_vec(r)?,
        },
        1 => Event::NoAmmo { player: player(r)? },
        2 => Event::WeaponSwitch {
            player: player(r)?,
            weapon: get_weapon(r)?,
        },
        3 => Event::HammerHit { pos: get_vec(r)? },
        4 => Event::LaserBounce { pos: get_vec(r)? },
        5 => Event::Explosion { pos: get_vec(r)? },
        6 => Event::Damage {
            player: player(r)?,
            from: get_opt(r)?,
            health: r.int("Schaden")?,
            armor: r.int("Schaden")?,
        },
        7 => {
            let (player, killer) = (player(r)?, get_opt(r)?);
            let cause = match r.u8()? {
                0xff => DeathCause::World,
                x => DeathCause::Weapon(
                    Weapon::ALL
                        .get(usize::from(x))
                        .copied()
                        .ok_or(DecodeError::Invalid("Waffe"))?,
                ),
            };
            Event::Death {
                player,
                killer,
                cause,
                pos: get_vec(r)?,
            }
        }
        8 => Event::Spawn {
            player: player(r)?,
            pos: get_vec(r)?,
        },
        9 => Event::Pickup {
            player: player(r)?,
            kind: get_pickup(r)?,
            pos: get_vec(r)?,
        },
        10 => Event::PickupRespawn {
            kind: get_pickup(r)?,
            pos: get_vec(r)?,
        },
        _ => return Err(DecodeError::Invalid("Ereignis")),
    })
}

impl ClientMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Join { version, name } => {
                w.u8(0);
                w.uvar(u64::from(*version));
                w.str(name);
            }
            Self::Input { ack, inputs } => {
                w.u8(1);
                w.uvar(ack.map_or(0, |t| t + 1));
                w.uvar(inputs.len() as u64);
                let first = inputs.first().map_or(0, |i| i.0);
                w.uvar(first);
                for (k, (tick, input)) in inputs.iter().enumerate() {
                    // Ticks aufeinanderfolgend: nur Abweichung vom erwarteten Tick
                    w.ivar(*tick as i64 - (first + k as u64) as i64);
                    put_input(&mut w, input);
                }
            }
            Self::Leave => w.u8(2),
        }
        w.into_bytes()
    }

    /// # Errors
    /// Bei fehlerhaften Daten.
    pub fn decode(data: &[u8]) -> DecodeResult<Self> {
        let mut r = Reader::new(data);
        let msg = match r.u8()? {
            0 => {
                let version = r.uint("Version")?;
                let name = r.str(MAX_NAME)?.to_owned();
                Self::Join { version, name }
            }
            1 => {
                let ack = r.uvar()?.checked_sub(1);
                let n: usize = r.uint("Anzahl")?;
                if n > MAX_INPUTS {
                    return Err(DecodeError::Invalid("Anzahl"));
                }
                let first = r.uvar()?;
                let mut inputs = Vec::with_capacity(n);
                for k in 0..n {
                    let off = r.ivar()?;
                    let tick = (first + k as u64)
                        .checked_add_signed(off)
                        .ok_or(DecodeError::Overflow)?;
                    inputs.push((tick, get_input(&mut r)?));
                }
                Self::Input { ack, inputs }
            }
            2 => Self::Leave,
            _ => return Err(DecodeError::Invalid("Nachricht")),
        };
        r.finish()?;
        Ok(msg)
    }
}

impl ServerMsg {
    /// Snapshot-Nachricht aus zwei Snapshots.
    pub fn snapshot(cur: &Snapshot, base: Option<&Snapshot>, events: Vec<Event>) -> Self {
        let mut w = Writer::new();
        cur.encode_delta(base, &mut w);
        Self::Snapshot {
            tick: cur.tick,
            base: base.map(|b| b.tick),
            checksum: cur.checksum(),
            delta: w.into_bytes(),
            events,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Welcome {
                slot,
                tick,
                map_name,
                map_source,
                tuning,
                high_bandwidth,
            } => {
                w.u8(0);
                w.uvar(u64::from(PROTOCOL_VERSION));
                w.uvar(u64::from(*slot));
                w.uvar(*tick);
                w.str(map_name);
                w.str(map_source);
                put_tuning(&mut w, tuning);
                w.bool(*high_bandwidth);
            }
            Self::Snapshot {
                tick,
                base,
                checksum,
                delta,
                events,
            } => {
                w.u8(1);
                w.uvar(*tick);
                w.uvar(base.map_or(0, |b| tick - b));
                w.uvar(u64::from(*checksum));
                w.bytes(delta);
                w.uvar(events.len() as u64);
                for e in events {
                    put_event(&mut w, e);
                }
            }
            Self::InputTiming { tick, time_left_ms } => {
                w.u8(2);
                w.uvar(*tick);
                w.ivar(i64::from(*time_left_ms));
            }
            Self::Tuning(t) => {
                w.u8(3);
                put_tuning(&mut w, t);
            }
            Self::Kick { reason } => {
                w.u8(4);
                w.str(reason);
            }
        }
        w.into_bytes()
    }

    /// # Errors
    /// Bei fehlerhaften Daten oder falscher Protokollversion.
    pub fn decode(data: &[u8]) -> DecodeResult<Self> {
        let mut r = Reader::new(data);
        let msg = match r.u8()? {
            0 => {
                if r.uvar()? != u64::from(PROTOCOL_VERSION) {
                    return Err(DecodeError::Invalid("Protokollversion"));
                }
                Self::Welcome {
                    slot: r.uint("Slot")?,
                    tick: r.uvar()?,
                    map_name: r.str(MAX_TEXT)?.to_owned(),
                    map_source: r.str(MAX_MAP)?.to_owned(),
                    tuning: get_tuning(&mut r)?,
                    high_bandwidth: r.bool()?,
                }
            }
            1 => {
                let tick = r.uvar()?;
                let back = r.uvar()?;
                let base = if back == 0 {
                    None
                } else {
                    Some(tick.checked_sub(back).ok_or(DecodeError::Overflow)?)
                };
                let checksum = r.uint("Prüfsumme")?;
                let delta = r.bytes(1 << 20)?.to_vec();
                let n: usize = r.uint("Anzahl")?;
                if n > MAX_EVENTS {
                    return Err(DecodeError::Invalid("Anzahl"));
                }
                let events = (0..n)
                    .map(|_| get_event(&mut r))
                    .collect::<DecodeResult<_>>()?;
                Self::Snapshot {
                    tick,
                    base,
                    checksum,
                    delta,
                    events,
                }
            }
            2 => Self::InputTiming {
                tick: r.uvar()?,
                time_left_ms: r.int("Zeit")?,
            },
            3 => Self::Tuning(get_tuning(&mut r)?),
            4 => Self::Kick {
                reason: r.str(MAX_TEXT)?.to_owned(),
            },
            _ => return Err(DecodeError::Invalid("Nachricht")),
        };
        r.finish()?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_messages_roundtrip() {
        let input = PlayerInput {
            direction: -1,
            target_x: -300,
            target_y: 20,
            jump: true,
            fire: 7,
            wanted_weapon: 3,
            ..PlayerInput::default()
        };
        for m in [
            ClientMsg::Join {
                version: PROTOCOL_VERSION,
                name: "Elora".into(),
            },
            ClientMsg::Input {
                ack: Some(1234),
                inputs: vec![(100, input), (101, PlayerInput::default()), (102, input)],
            },
            ClientMsg::Input {
                ack: None,
                inputs: vec![],
            },
            ClientMsg::Leave,
        ] {
            assert_eq!(ClientMsg::decode(&m.encode()).unwrap(), m);
        }
    }

    #[test]
    fn server_messages_roundtrip() {
        let events = vec![
            Event::Explosion {
                pos: Vec2::new(10.0, -20.0),
            },
            Event::Death {
                player: 3,
                killer: Some(1),
                cause: DeathCause::Weapon(Weapon::Laser),
                pos: Vec2::new(1.0, 2.0),
            },
            Event::Death {
                player: 3,
                killer: None,
                cause: DeathCause::World,
                pos: Vec2::ZERO,
            },
            Event::Pickup {
                player: 0,
                kind: PickupKind::Weapon(Weapon::Grenade),
                pos: Vec2::ZERO,
            },
            Event::Damage {
                player: 2,
                from: None,
                health: 3,
                armor: 1,
            },
        ];
        for m in [
            ServerMsg::Welcome {
                slot: 2,
                tick: 99,
                map_name: "sandbox".into(),
                map_source: "format = 1".into(),
                tuning: Tuning::default(),
                high_bandwidth: true,
            },
            ServerMsg::Snapshot {
                tick: 50,
                base: Some(48),
                checksum: 7,
                delta: vec![1, 2, 3],
                events,
            },
            ServerMsg::Snapshot {
                tick: 50,
                base: None,
                checksum: 0,
                delta: vec![],
                events: vec![],
            },
            ServerMsg::InputTiming {
                tick: 12,
                time_left_ms: -5,
            },
            ServerMsg::Tuning(Tuning::default()),
            ServerMsg::Kick {
                reason: "Server voll".into(),
            },
        ] {
            assert_eq!(ServerMsg::decode(&m.encode()).unwrap(), m);
        }
    }

    #[test]
    fn rejects_invalid_input() {
        let mut bytes = ClientMsg::Input {
            ack: None,
            inputs: vec![(1, PlayerInput::default())],
        }
        .encode();
        // Richtung manipulieren (Position nach Tag, ack, Anzahl, first, offset)
        bytes[5] = 4; // ZigZag 2
        assert!(ClientMsg::decode(&bytes).is_err());
        assert!(ClientMsg::decode(&[9]).is_err());
        assert!(ServerMsg::decode(&[]).is_err());
    }
}
