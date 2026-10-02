//! Welche Sounds Ereignisse auslösen (M5.7). Reine Zuordnung ohne Wiedergabe –
//! dieselben Ereignisse speisen auch Effekte und Killfeed.

use elora_sim::character::events as bits;
use elora_sim::{Event, HookState, PickupKind, Team, Vec2, Weapon};

/// Alle Sounds des Spiels. Die Namen (`snake_case`) sind die Schlüssel in
/// `assets/sounds/sounds.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sound {
    HammerFire,
    HammerHit,
    GrenadeFire,
    GrenadeExplode,
    LaserFire,
    LaserBounce,
    NoAmmo,
    WeaponSwitch,
    HookFire,
    HookAttachGround,
    HookAttachPlayer,
    HookNoAttach,
    Jump,
    AirJump,
    Land,
    PainShort,
    PainLong,
    Death,
    Spawn,
    PickupHealth,
    PickupArmor,
    PickupWeapon,
    PickupRespawn,
    /// Eigener Treffer (Treffer-Bestätigung).
    HitConfirm,
    Chat,
    Emote,
    /// Die eigene Flagge wurde genommen.
    FlagGrabOwn,
    /// Das eigene Team hat die gegnerische Flagge.
    FlagGrabEnemy,
    FlagDrop,
    FlagReturn,
    FlagCapture,
}

impl Sound {
    pub const ALL: [Self; 31] = [
        Self::HammerFire,
        Self::HammerHit,
        Self::GrenadeFire,
        Self::GrenadeExplode,
        Self::LaserFire,
        Self::LaserBounce,
        Self::NoAmmo,
        Self::WeaponSwitch,
        Self::HookFire,
        Self::HookAttachGround,
        Self::HookAttachPlayer,
        Self::HookNoAttach,
        Self::Jump,
        Self::AirJump,
        Self::Land,
        Self::PainShort,
        Self::PainLong,
        Self::Death,
        Self::Spawn,
        Self::PickupHealth,
        Self::PickupArmor,
        Self::PickupWeapon,
        Self::PickupRespawn,
        Self::HitConfirm,
        Self::Chat,
        Self::Emote,
        Self::FlagGrabOwn,
        Self::FlagGrabEnemy,
        Self::FlagDrop,
        Self::FlagReturn,
        Self::FlagCapture,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::HammerFire => "hammer_fire",
            Self::HammerHit => "hammer_hit",
            Self::GrenadeFire => "grenade_fire",
            Self::GrenadeExplode => "grenade_explode",
            Self::LaserFire => "laser_fire",
            Self::LaserBounce => "laser_bounce",
            Self::NoAmmo => "no_ammo",
            Self::WeaponSwitch => "weapon_switch",
            Self::HookFire => "hook_fire",
            Self::HookAttachGround => "hook_attach_ground",
            Self::HookAttachPlayer => "hook_attach_player",
            Self::HookNoAttach => "hook_no_attach",
            Self::Jump => "jump",
            Self::AirJump => "air_jump",
            Self::Land => "land",
            Self::PainShort => "pain_short",
            Self::PainLong => "pain_long",
            Self::Death => "death",
            Self::Spawn => "spawn",
            Self::PickupHealth => "pickup_health",
            Self::PickupArmor => "pickup_armor",
            Self::PickupWeapon => "pickup_weapon",
            Self::PickupRespawn => "pickup_respawn",
            Self::HitConfirm => "hit_confirm",
            Self::Chat => "chat",
            Self::Emote => "emote",
            Self::FlagGrabOwn => "flag_grab_own",
            Self::FlagGrabEnemy => "flag_grab_enemy",
            Self::FlagDrop => "flag_drop",
            Self::FlagReturn => "flag_return",
            Self::FlagCapture => "flag_capture",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// Ein abzuspielender Sound; `pos = None`: nicht räumlich (Hinweise, UI).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cue {
    pub sound: Sound,
    pub pos: Option<Vec2>,
}

impl Cue {
    pub fn at(sound: Sound, pos: Vec2) -> Self {
        Self {
            sound,
            pos: Some(pos),
        }
    }

    pub fn global(sound: Sound) -> Self {
        Self { sound, pos: None }
    }
}

/// Wer zuhört: eigener Slot und eigenes Team (für Treffer-Bestätigung und CTF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Listener {
    pub local: Option<usize>,
    pub team: Team,
}

/// Ab so viel Schaden (Leben + Rüstung) der lange Schmerzlaut.
const PAIN_LONG_FROM: i32 = 3;

/// Sounds für ein Simulations-Ereignis; `pos_of` liefert die Position einer Figur.
pub fn for_event(e: &Event, l: Listener, pos_of: impl Fn(usize) -> Option<Vec2>) -> Vec<Cue> {
    let at_player = |sound, player| pos_of(player).map(|p| Cue::at(sound, p));
    match *e {
        Event::Fire { weapon, pos, .. } => vec![Cue::at(
            match weapon {
                Weapon::Hammer => Sound::HammerFire,
                Weapon::Grenade => Sound::GrenadeFire,
                Weapon::Laser => Sound::LaserFire,
            },
            pos,
        )],
        Event::NoAmmo { player } => at_player(Sound::NoAmmo, player).into_iter().collect(),
        Event::WeaponSwitch { player, .. } => {
            at_player(Sound::WeaponSwitch, player).into_iter().collect()
        }
        // Stampfen (A1.1) klingt vorerst wie ein Hammertreffer; eigene Sounds liefert der Projektinhaber (E-109)
        Event::HammerHit { pos, .. } | Event::Stomp { pos, .. } => {
            vec![Cue::at(Sound::HammerHit, pos)]
        }
        Event::LaserBounce { pos, .. } => vec![Cue::at(Sound::LaserBounce, pos)],
        Event::Explosion { pos, .. } => vec![Cue::at(Sound::GrenadeExplode, pos)],
        Event::Damage {
            player,
            from,
            health,
            armor,
        } => {
            let pain = if health + armor >= PAIN_LONG_FROM {
                Sound::PainLong
            } else {
                Sound::PainShort
            };
            let mut cues: Vec<Cue> = at_player(pain, player).into_iter().collect();
            if from.is_some_and(|f| Some(f) == l.local && f != player) {
                cues.push(Cue::global(Sound::HitConfirm));
            }
            cues
        }
        Event::Death { pos, .. } | Event::CreatureDeath { pos, .. } => {
            vec![Cue::at(Sound::Death, pos)]
        }
        Event::Spawn { pos, .. } => vec![Cue::at(Sound::Spawn, pos)],
        Event::Pickup { kind, pos, .. } => vec![Cue::at(
            match kind {
                PickupKind::Health => Sound::PickupHealth,
                PickupKind::Armor => Sound::PickupArmor,
                PickupKind::Weapon(_) => Sound::PickupWeapon,
            },
            pos,
        )],
        Event::PickupRespawn { pos, .. } => vec![Cue::at(Sound::PickupRespawn, pos)],
        Event::FlagGrab { team, .. } => vec![Cue::global(if team == l.team {
            Sound::FlagGrabOwn
        } else {
            Sound::FlagGrabEnemy
        })],
        Event::FlagDrop { .. } => vec![Cue::global(Sound::FlagDrop)],
        Event::FlagReturn { .. } => vec![Cue::global(Sound::FlagReturn)],
        Event::FlagCapture { .. } => vec![Cue::global(Sound::FlagCapture)],
        Event::TileBroken { .. } => Vec::new(),
        // Gegner (A1.2): vorerst vorhandene Sounds, eigene liefert der Projektinhaber (E-109)
        Event::CreatureHit { pos, from, .. } => {
            let mut cues = vec![Cue::at(Sound::PainShort, pos)];
            if from.is_some() && from == l.local {
                cues.push(Cue::global(Sound::HitConfirm));
            }
            cues
        }
        Event::CreatureFire { pos, .. } => vec![Cue::at(Sound::HookFire, pos)],
        Event::LootCollect { pos, .. } => vec![Cue::at(Sound::PickupArmor, pos)],
    }
}

/// Sounds aus dem Zustand einer Figur: Sprünge und Hook (Bits des letzten Ticks)
/// sowie Hook-Abschuss (Wechsel nach [`HookState::Flying`]).
pub fn for_character(pos: Vec2, triggered: u16, prev_hook: HookState, hook: HookState) -> Vec<Cue> {
    let mut cues = Vec::new();
    let table = [
        (bits::GROUND_JUMP, Sound::Jump),
        (bits::AIR_JUMP, Sound::AirJump),
        (bits::HOOK_ATTACH_GROUND, Sound::HookAttachGround),
        (bits::HOOK_ATTACH_PLAYER, Sound::HookAttachPlayer),
        (bits::HOOK_HIT_UNHOOKABLE, Sound::HookNoAttach),
    ];
    for (bit, sound) in table {
        if triggered & bit != 0 {
            cues.push(Cue::at(sound, pos));
        }
    }
    if hook == HookState::Flying && prev_hook != HookState::Flying {
        cues.push(Cue::at(Sound::HookFire, pos));
    }
    cues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_roundtrip() {
        let mut names: Vec<&str> = Sound::ALL.iter().map(|s| s.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Sound::ALL.len());
        for s in Sound::ALL {
            assert_eq!(Sound::from_name(s.name()), Some(s));
        }
    }

    #[test]
    fn damage_pain_and_hit_confirm() {
        let l = Listener {
            local: Some(1),
            team: Team::Red,
        };
        let pos = |_| Some(Vec2::new(5.0, 5.0));
        let hit = |health, from| Event::Damage {
            player: 2,
            from,
            health,
            armor: 0,
        };
        let cues = for_event(&hit(1, Some(1)), l, pos);
        assert_eq!(cues[0].sound, Sound::PainShort);
        assert_eq!(cues[1], Cue::global(Sound::HitConfirm));
        let cues = for_event(&hit(4, Some(3)), l, pos);
        assert_eq!(cues, vec![Cue::at(Sound::PainLong, Vec2::new(5.0, 5.0))]);
    }

    #[test]
    fn ctf_depends_on_own_team() {
        let grab = |team| Event::FlagGrab {
            team,
            player: 0,
            from_stand: true,
        };
        let l = Listener {
            local: Some(0),
            team: Team::Blue,
        };
        assert_eq!(
            for_event(&grab(Team::Blue), l, |_| None)[0].sound,
            Sound::FlagGrabOwn
        );
        assert_eq!(
            for_event(&grab(Team::Red), l, |_| None)[0].sound,
            Sound::FlagGrabEnemy
        );
    }

    #[test]
    fn character_bits_and_hook_launch() {
        let p = Vec2::default();
        let cues = for_character(
            p,
            bits::GROUND_JUMP | bits::HOOK_ATTACH_GROUND,
            HookState::Idle,
            HookState::Idle,
        );
        let sounds: Vec<Sound> = cues.iter().map(|c| c.sound).collect();
        assert_eq!(sounds, [Sound::Jump, Sound::HookAttachGround]);
        let cues = for_character(p, 0, HookState::Idle, HookState::Flying);
        assert_eq!(cues[0].sound, Sound::HookFire);
        assert!(for_character(p, 0, HookState::Flying, HookState::Flying).is_empty());
    }
}
