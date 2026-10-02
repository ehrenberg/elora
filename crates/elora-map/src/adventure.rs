//! Abenteuer-Objekte einer Karte (R2-M1, A1.5, E-252 bis E-259): Gegner, NPCs, Truhen,
//! Schalter, Türen, Sammelstücke, Speicherpunkte, Heilpflanzen, Eingänge, Übergänge, Zonen
//! und Kamera-Zonen. Abschnitt `ADVN` im Kartenformat; Mehrspieler-Karten haben keinen.
//!
//! Die Karte prüft nur den Aufbau (eindeutige Ids, Lage, Größen); ob Gegnerarten, Gespräche
//! oder Zielkarten existieren, prüft das Abenteuer (`elora-adventure`).

use elora_sim::{TILE_SIZE, Vec2};

/// Höchstzahl der Objekte einer Karte.
pub const MAX_OBJECTS: usize = 4096;
/// Höchstzahl der Einträge in Listen (Truhen-Inhalt).
pub const MAX_LIST: usize = 64;

/// Wie ein Schalter ausgelöst wird (E-256).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwitchTrigger {
    /// Aktionstaste.
    #[default]
    Interact,
    /// Hammer-Treffer.
    Hammer,
    /// Heranhooken.
    Hook,
}

/// Kamera-Zone (E-259).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    /// Fester Ausschnitt: die Kamera zeigt die Zone.
    #[default]
    Fixed,
    /// Begrenzen: die Kamera bleibt innerhalb der Zone.
    Bounds,
}

/// Art eines Objekts mit seinen Einstellungen.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    /// Gegner (`creatures.toml`); `persistent` = Boss/besonders, bleibt besiegt (E-235).
    Creature { kind: String, persistent: bool },
    /// Figur mit Gespräch (E-257); `walk` = halbe Länge des Laufwegs (0 = steht).
    Npc {
        character: String,
        dialog: String,
        facing: i8,
        walk: f32,
    },
    /// Truhe mit festem Inhalt (E-255); `lock` = Bedingung zum Öffnen (leer = offen).
    Chest {
        contents: Vec<(String, u32)>,
        lock: String,
    },
    /// Hebel oder Treffer-Schalter (E-256): setzt `flag` auf 1/0.
    Switch {
        flag: String,
        once: bool,
        trigger: SwitchTrigger,
    },
    /// Tür/Tor (E-254): Block aus `size` (Tiles, ab `pos` als linker oberer Ecke), zu wie
    /// Stein, offen sobald `open_if` gilt.
    Door { size: (u8, u8), open_if: String },
    /// Einmaliger Fund (z. B. Glitzerstein).
    Collectible { item: String },
    /// Quellstein (P-31).
    SavePoint,
    /// Heilpflanze (E-258), wächst beim Wiederbetreten nach.
    HealPlant { heal: i32 },
    /// Eingang: hier erscheint Elora, wenn ein Übergang mit dieser Id ankommt.
    Spawn,
    /// Übergang zu `map`/`spawn` (E-252); Bereich `size` ab `pos`. `on_touch` = beim
    /// Hineinlaufen, sonst mit der Aktionstaste.
    Exit {
        size: Vec2,
        map: String,
        spawn: String,
        on_touch: bool,
    },
    /// Auslöser-Zone für Aufgaben („Ort erreichen“); Bereich `size` ab `pos`.
    Zone { size: Vec2 },
    /// Kamera-Zone; Bereich `size` ab `pos`.
    Camera { size: Vec2, mode: CameraMode },
}

impl ObjectKind {
    /// Kennung für Editor und Fehlermeldungen.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Creature { .. } => "gegner",
            Self::Npc { .. } => "npc",
            Self::Chest { .. } => "truhe",
            Self::Switch { .. } => "schalter",
            Self::Door { .. } => "tuer",
            Self::Collectible { .. } => "sammelstueck",
            Self::SavePoint => "speicherpunkt",
            Self::HealPlant { .. } => "heilpflanze",
            Self::Spawn => "eingang",
            Self::Exit { .. } => "uebergang",
            Self::Zone { .. } => "zone",
            Self::Camera { .. } => "kamera",
        }
    }

    /// Größe eines Bereichs (Übergang, Zone, Kamera, Tür) in Einheiten.
    pub fn area(&self) -> Option<Vec2> {
        match self {
            Self::Exit { size, .. } | Self::Zone { size } | Self::Camera { size, .. } => {
                Some(*size)
            }
            Self::Door { size, .. } => {
                #[allow(clippy::cast_precision_loss)]
                let ts = TILE_SIZE as f32;
                Some(Vec2::new(f32::from(size.0) * ts, f32::from(size.1) * ts))
            }
            _ => None,
        }
    }
}

/// Ein Objekt. `id` ist auf der Karte eindeutig und Schlüssel im Spielstand
/// (z. B. geöffnete Truhe `wiese-1:truhe-3`).
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub id: String,
    /// Mitte (Figuren, Gegenstände) bzw. linke obere Ecke (Bereiche, Türen) in Einheiten.
    pub pos: Vec2,
    pub kind: ObjectKind,
}

/// Abenteuer-Teil einer Karte.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Adventure {
    pub objects: Vec<Object>,
}

impl Adventure {
    pub fn object(&self, id: &str) -> Option<&Object> {
        self.objects.iter().find(|o| o.id == id)
    }

    /// Aufbau prüfen (Karte `w`×`h` Tiles).
    ///
    /// # Errors
    /// Mit dem ersten Fehler und der Id des Objekts.
    pub fn validate(&self, w: usize, h: usize) -> Result<(), String> {
        #[allow(clippy::cast_precision_loss)]
        let (ts, mw, mh) = (
            TILE_SIZE as f32,
            w as f32 * TILE_SIZE as f32,
            h as f32 * TILE_SIZE as f32,
        );
        if self.objects.len() > MAX_OBJECTS {
            return Err(format!("mehr als {MAX_OBJECTS} Abenteuer-Objekte"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for o in &self.objects {
            let at = |m: &str| Err(format!("Objekt `{}` ({}): {m}", o.id, o.kind.name()));
            if o.id.is_empty() || o.id.len() > crate::binary::MAX_NAME || o.id.contains(':') {
                return at("Id leer, zu lang oder mit `:`");
            }
            if !ids.insert(o.id.as_str()) {
                return at("Id doppelt");
            }
            let inside = |p: Vec2| p.x >= 0.0 && p.y >= 0.0 && p.x <= mw && p.y <= mh;
            if !o.pos.x.is_finite() || !o.pos.y.is_finite() || !inside(o.pos) {
                return at("liegt außerhalb der Karte");
            }
            if let Some(size) = o.kind.area()
                && (size.x <= 0.0 || size.y <= 0.0 || !inside(o.pos + size))
            {
                return at("Bereich leer oder außerhalb der Karte");
            }
            match &o.kind {
                ObjectKind::Door { .. } if o.pos.x % ts != 0.0 || o.pos.y % ts != 0.0 => {
                    return at("Tür liegt nicht auf dem Tile-Raster");
                }
                ObjectKind::Chest { contents, .. } if contents.len() > MAX_LIST => {
                    return at("zu viele Gegenstände");
                }
                ObjectKind::Npc { facing, walk, .. }
                    if !matches!(facing, -1 | 1) || *walk < 0.0 =>
                {
                    return at("Blickrichtung oder Laufweg ungültig");
                }
                ObjectKind::Exit { map, spawn, .. } if map.is_empty() || spawn.is_empty() => {
                    return at("Ziel fehlt");
                }
                _ => {}
            }
        }
        Ok(())
    }
}
