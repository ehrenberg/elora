//! Release-Kartenformat `.emap` (E-129, E-143 bis E-146), beschrieben in `docs/05-kartenformat.md`.
//!
//! Datei: `EMAP` + Formatversion (u16) + zlib-komprimierte Abschnitte. Jeder Abschnitt ist
//! `Kennung (4 Byte) | Länge (u32) | Inhalt`. Zahlen sind Little Endian.

use elora_sim::{BeltDir, DummyPattern, JumpDir, Tile, Vec2};

use crate::look::{
    Art, Background, Curve, Decor, EnvKind, EnvPoint, EnvRef, Envelope, Image, Rgba, Sky,
};
use crate::{Entity, EntityKind, MAX_SIZE, Map};

/// Kennung am Dateianfang.
pub const MAGIC: [u8; 4] = *b"EMAP";
/// Unterstützte Version des Binärformats.
pub const FORMAT_VERSION: u16 = 1;

/// Höchstens so viele Bytes werden entpackt (Schutz vor Zip-Bomben).
pub const MAX_PAYLOAD: usize = 32 << 20;
/// Größe eines eingebetteten SVGs.
pub const MAX_IMAGE_BYTES: usize = 512 << 10;
pub const MAX_IMAGES: usize = 64;
pub const MAX_MATERIALS: usize = 255;
pub const MAX_BACKGROUNDS: usize = 16;
/// Deko-Objekte insgesamt (alle Ebenen).
pub const MAX_DECOR: usize = 20_000;
pub const MAX_ENVELOPES: usize = 256;
pub const MAX_ENV_POINTS: usize = 1024;
/// Länge von Namen und Kennungen in Bytes.
pub const MAX_NAME: usize = 128;

/// Fehler beim Lesen einer Karte.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MapError {
    #[error("keine Elora-Karte (Kennung fehlt)")]
    BadMagic,
    #[error("nicht unterstützte Formatversion {found} (unterstützt: {FORMAT_VERSION})")]
    UnsupportedFormat { found: u16 },
    #[error("Kartendaten sind beschädigt (Entpacken fehlgeschlagen)")]
    Compression,
    #[error("Kartendaten sind zu groß (über {MAX_PAYLOAD} Bytes entpackt)")]
    PayloadTooLarge,
    #[error("Kartendaten enden zu früh")]
    Truncated,
    #[error("Abschnitt `{0}` fehlt")]
    MissingSection(&'static str),
    #[error("Abschnitt `{0}` kommt doppelt vor")]
    DuplicateSection(String),
    #[error("ungültige Kartendaten: {0}")]
    Invalid(&'static str),
    #[error("Raster {width}×{height} ist größer als erlaubt ({MAX_SIZE}×{MAX_SIZE})")]
    TooLarge { width: usize, height: usize },
    #[error("Zeile {line}: Länge {found}, erwartet {expected}")]
    RaggedRow {
        line: usize,
        found: usize,
        expected: usize,
    },
    #[error("Zeile {line}, Spalte {column}: unbekanntes Zeichen `{symbol}`")]
    UnknownSymbol {
        line: usize,
        column: usize,
        symbol: char,
    },
    #[error("Karte hat keinen Spawnpunkt")]
    NoSpawn,
    #[error("Flaggen: {red}× rot, {blue}× blau – für CTF genau je eine, sonst keine")]
    InvalidFlags { red: usize, blue: usize },
}

type Result<T> = std::result::Result<T, MapError>;

// ---------------------------------------------------------------- Kodierung der Aufzählungen

const TILES: [Tile; 11] = [
    Tile::Air,
    Tile::Solid,
    Tile::Unhookable,
    Tile::Death,
    Tile::Platform,
    Tile::Ice,
    Tile::JumpPad(JumpDir::Up),
    Tile::JumpPad(JumpDir::UpLeft),
    Tile::JumpPad(JumpDir::UpRight),
    Tile::Conveyor(BeltDir::Left),
    Tile::Conveyor(BeltDir::Right),
];

const ENTITIES: [EntityKind; 13] = [
    EntityKind::Spawn,
    EntityKind::SpawnRed,
    EntityKind::SpawnBlue,
    EntityKind::FlagRed,
    EntityKind::FlagBlue,
    EntityKind::Health,
    EntityKind::Armor,
    EntityKind::Laser,
    EntityKind::Grenade,
    EntityKind::Dummy(DummyPattern::Stand),
    EntityKind::Dummy(DummyPattern::Walk),
    EntityKind::Dummy(DummyPattern::Jump),
    EntityKind::Dummy(DummyPattern::WalkJump),
];

const CURVES: [Curve; 5] = [
    Curve::Step,
    Curve::Linear,
    Curve::Slow,
    Curve::Fast,
    Curve::Smooth,
];

fn code<T: PartialEq>(table: &[T], v: &T) -> u8 {
    let i = table
        .iter()
        .position(|t| t == v)
        .expect("Tabelle vollständig");
    u8::try_from(i).expect("Tabelle < 256")
}

fn lookup<T: Copy>(table: &[T], c: u8, what: &'static str) -> Result<T> {
    table
        .get(usize::from(c))
        .copied()
        .ok_or(MapError::Invalid(what))
}

// ---------------------------------------------------------------- Schreiben

#[derive(Default)]
struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn vec2(&mut self, v: Vec2) {
        self.f32(v.x);
        self.f32(v.y);
    }
    fn len(&mut self, n: usize) {
        self.u32(u32::try_from(n).expect("Länge passt in u32"));
    }
    fn bytes(&mut self, v: &[u8]) {
        self.len(v.len());
        self.0.extend_from_slice(v);
    }
    fn str(&mut self, v: &str) {
        self.bytes(v.as_bytes());
    }
    fn rgba(&mut self, c: Rgba) {
        self.0.extend_from_slice(&c.0);
    }
    fn section(&mut self, tag: [u8; 4], body: &Writer) {
        self.0.extend_from_slice(&tag);
        self.bytes(&body.0);
    }
}

/// Schreibt eine Karte ins Binärformat.
///
/// # Panics
/// Wenn die Karte die Grenzen des Formats sprengt (z. B. über 4 GB) – [`decode`] lehnt solche Karten ab.
pub fn encode(map: &Map) -> Vec<u8> {
    let mut payload = Writer::default();

    let mut s = Writer::default();
    s.str(&map.name);
    s.str(map.author.as_deref().unwrap_or(""));
    payload.section(*b"INFO", &s);

    let mut s = Writer::default();
    s.len(map.width);
    s.len(map.height);
    for t in &map.tiles {
        s.u8(code(&TILES, t));
    }
    payload.section(*b"GAME", &s);

    let mut s = Writer::default();
    s.len(map.entities.len());
    for e in &map.entities {
        s.u8(code(&ENTITIES, &e.kind));
        s.len(e.tx);
        s.len(e.ty);
    }
    payload.section(*b"ENTS", &s);

    if !map.materials.is_empty() {
        let mut s = Writer::default();
        s.len(map.materials.len());
        for m in &map.materials {
            s.str(m);
        }
        s.bytes(&map.material_map);
        payload.section(*b"MATL", &s);
    }

    let mut s = Writer::default();
    s.rgba(map.sky.top);
    s.rgba(map.sky.bottom);
    payload.section(*b"SKY ", &s);

    if !map.backgrounds.is_empty() {
        let mut s = Writer::default();
        s.len(map.backgrounds.len());
        for b in &map.backgrounds {
            s.str(&b.name);
            s.vec2(b.parallax);
            s.vec2(b.offset);
            s.f32(b.repeat_x.unwrap_or(0.0));
            put_decor_list(&mut s, &b.items);
        }
        payload.section(*b"BGRD", &s);
    }

    if !map.decor_back.is_empty() || !map.decor_front.is_empty() {
        let mut s = Writer::default();
        put_decor_list(&mut s, &map.decor_back);
        put_decor_list(&mut s, &map.decor_front);
        payload.section(*b"DECO", &s);
    }

    if !map.envelopes.is_empty() {
        let mut s = Writer::default();
        s.len(map.envelopes.len());
        for e in &map.envelopes {
            s.str(&e.name);
            s.u8(match e.kind {
                EnvKind::Position => 0,
                EnvKind::Color => 1,
            });
            s.u8(u8::from(e.synced));
            s.len(e.points.len());
            for p in &e.points {
                s.u32(p.time_ms);
                p.value.iter().for_each(|v| s.f32(*v));
                s.u8(code(&CURVES, &p.curve));
            }
        }
        payload.section(*b"ENVL", &s);
    }

    if !map.images.is_empty() {
        let mut s = Writer::default();
        s.len(map.images.len());
        for i in &map.images {
            s.str(&i.name);
            s.bytes(&i.svg);
        }
        payload.section(*b"IMGS", &s);
    }

    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(&payload.0, 9));
    out
}

fn put_env_ref(w: &mut Writer, r: Option<EnvRef>) {
    match r {
        None => w.u16(u16::MAX),
        Some(r) => {
            w.u16(r.index);
            w.i32(r.offset_ms);
        }
    }
}

fn put_decor_list(w: &mut Writer, items: &[Decor]) {
    w.len(items.len());
    for d in items {
        match &d.art {
            Art::Builtin(name) => {
                w.u8(0);
                w.str(name);
            }
            Art::Image(i) => {
                w.u8(1);
                w.u16(*i);
            }
        }
        w.vec2(d.pos);
        w.f32(d.scale);
        w.f32(d.rotation);
        w.u8(u8::from(d.flip_x));
        w.rgba(d.tint);
        put_env_ref(w, d.pos_env);
        put_env_ref(w, d.color_env);
    }
}

// ---------------------------------------------------------------- Lesen

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(MapError::Truncated);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("Länge stimmt"))
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(MapError::Invalid("Wahrheitswert")),
        }
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    fn f32(&mut self) -> Result<f32> {
        let v = f32::from_le_bytes(self.array()?);
        if v.is_finite() {
            Ok(v)
        } else {
            Err(MapError::Invalid("Zahl ist nicht endlich"))
        }
    }
    fn vec2(&mut self) -> Result<Vec2> {
        Ok(Vec2::new(self.f32()?, self.f32()?))
    }
    /// Anzahl/Länge mit Obergrenze.
    fn len(&mut self, max: usize, what: &'static str) -> Result<usize> {
        let n = usize::try_from(self.u32()?).map_err(|_| MapError::Invalid(what))?;
        if n > max {
            return Err(MapError::Invalid(what));
        }
        Ok(n)
    }
    fn bytes(&mut self, max: usize, what: &'static str) -> Result<&'a [u8]> {
        let n = self.len(max, what)?;
        self.take(n)
    }
    fn str(&mut self, max: usize, what: &'static str) -> Result<String> {
        let b = self.bytes(max, what)?;
        Ok(std::str::from_utf8(b)
            .map_err(|_| MapError::Invalid(what))?
            .to_owned())
    }
    fn rgba(&mut self) -> Result<Rgba> {
        Ok(Rgba(self.array()?))
    }
    fn done(&self, what: &'static str) -> Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(MapError::Invalid(what))
        }
    }
}

/// Die Abschnitte einer Datei, nach Kennung.
#[derive(Default)]
struct Sections<'a> {
    found: Vec<([u8; 4], &'a [u8])>,
}

impl<'a> Sections<'a> {
    fn parse(mut r: Reader<'a>) -> Result<Self> {
        let mut s = Self::default();
        while !r.0.is_empty() {
            let tag = r.array::<4>()?;
            let body = r.bytes(MAX_PAYLOAD, "Abschnittslänge")?;
            if s.found.iter().any(|(t, _)| *t == tag) {
                return Err(MapError::DuplicateSection(
                    String::from_utf8_lossy(&tag).into_owned(),
                ));
            }
            // Unbekannte Abschnitte bleiben liegen (spätere Erweiterungen derselben Version)
            s.found.push((tag, body));
        }
        Ok(s)
    }

    fn get(&self, tag: [u8; 4]) -> Option<Reader<'a>> {
        self.found
            .iter()
            .find(|(t, _)| *t == tag)
            .map(|(_, b)| Reader(b))
    }

    /// `&'static`, damit die Kennung im Fehler steht.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn require(&self, tag: &'static [u8; 4]) -> Result<Reader<'a>> {
        self.get(*tag).ok_or(MapError::MissingSection(
            std::str::from_utf8(tag).expect("Kennung ist ASCII"),
        ))
    }
}

/// Liest eine Karte aus dem Binärformat und prüft sie.
///
/// # Errors
/// Bei falscher Kennung oder Version, beschädigten Daten, überschrittenen Grenzen oder einer
/// unspielbaren Karte (kein Spawn, falsche Flaggen).
pub fn decode(data: &[u8]) -> Result<Map> {
    let map = decode_draft(data)?;
    validate(&map)?;
    Ok(map)
}

/// Wie [`decode`], aber ohne die Spielbarkeits-Prüfung (Spawn, Flaggen) – für Entwürfe im Editor.
/// Alle Schutzgrenzen und Verweise werden weiterhin geprüft.
///
/// # Errors
/// Bei falscher Kennung oder Version, beschädigten Daten oder überschrittenen Grenzen.
pub fn decode_draft(data: &[u8]) -> Result<Map> {
    let mut head = Reader(data);
    if head.array::<4>().map_err(|_| MapError::BadMagic)? != MAGIC {
        return Err(MapError::BadMagic);
    }
    let version = head.u16()?;
    if version != FORMAT_VERSION {
        return Err(MapError::UnsupportedFormat { found: version });
    }
    let payload = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(head.0, MAX_PAYLOAD)
        .map_err(|e| match e.status {
            miniz_oxide::inflate::TINFLStatus::HasMoreOutput => MapError::PayloadTooLarge,
            _ => MapError::Compression,
        })?;
    let sections = Sections::parse(Reader(&payload))?;

    let mut r = sections.require(b"INFO")?;
    let name = r.str(MAX_NAME, "Kartenname")?;
    let author = r.str(MAX_NAME, "Autor")?;
    r.done("INFO")?;

    let mut r = sections.require(b"GAME")?;
    let width = r.len(usize::MAX, "Breite")?;
    let height = r.len(usize::MAX, "Höhe")?;
    if width == 0 || height == 0 {
        return Err(MapError::Invalid("leeres Raster"));
    }
    if width > MAX_SIZE || height > MAX_SIZE {
        return Err(MapError::TooLarge { width, height });
    }
    let tiles = r
        .take(width * height)?
        .iter()
        .map(|&c| lookup(&TILES, c, "Tile-Art"))
        .collect::<Result<Vec<_>>>()?;
    r.done("GAME")?;

    let mut r = sections.require(b"ENTS")?;
    let n = r.len(width * height, "Anzahl Entities")?;
    let mut entities = Vec::with_capacity(n);
    for _ in 0..n {
        let kind = lookup(&ENTITIES, r.u8()?, "Entity-Art")?;
        let tx = r.len(width - 1, "Entity außerhalb")?;
        let ty = r.len(height - 1, "Entity außerhalb")?;
        entities.push(Entity { kind, tx, ty });
    }
    r.done("ENTS")?;

    let mut map = Map {
        name,
        author: (!author.is_empty()).then_some(author),
        width,
        height,
        tiles,
        entities,
        ..Map::new("", 0, 0)
    };
    decode_look(&sections, &mut map)?;
    check_references(&map)?;
    Ok(map)
}

/// Optionale Abschnitte zum Aussehen (Materialien, Himmel, Ebenen, Deko, Animationen, Bilder).
fn decode_look(sections: &Sections<'_>, map: &mut Map) -> Result<()> {
    let (width, height) = (map.width, map.height);
    let (materials, material_map) = match sections.get(*b"MATL") {
        None => (Vec::new(), Vec::new()),
        Some(mut r) => {
            let n = r.len(MAX_MATERIALS, "Anzahl Materialien")?;
            let materials = (0..n)
                .map(|_| r.str(MAX_NAME, "Material"))
                .collect::<Result<Vec<_>>>()?;
            let map = r.bytes(width * height, "Materialraster")?.to_vec();
            if map.len() != width * height || map.iter().any(|&m| usize::from(m) > n) {
                return Err(MapError::Invalid("Materialraster"));
            }
            r.done("MATL")?;
            (materials, map)
        }
    };

    let sky = match sections.get(*b"SKY ") {
        None => Sky::default(),
        Some(mut r) => {
            let sky = Sky {
                top: r.rgba()?,
                bottom: r.rgba()?,
            };
            r.done("SKY")?;
            sky
        }
    };

    let mut decor_budget = MAX_DECOR;
    let backgrounds = match sections.get(*b"BGRD") {
        None => Vec::new(),
        Some(mut r) => {
            let n = r.len(MAX_BACKGROUNDS, "Anzahl Hintergrund-Ebenen")?;
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                let name = r.str(MAX_NAME, "Ebenenname")?;
                let parallax = r.vec2()?;
                let offset = r.vec2()?;
                let repeat = r.f32()?;
                if repeat < 0.0 {
                    return Err(MapError::Invalid("Wiederholung"));
                }
                let items = get_decor_list(&mut r, &mut decor_budget)?;
                list.push(Background {
                    name,
                    parallax,
                    offset,
                    repeat_x: (repeat > 0.0).then_some(repeat),
                    items,
                });
            }
            r.done("BGRD")?;
            list
        }
    };

    let (decor_back, decor_front) = match sections.get(*b"DECO") {
        None => (Vec::new(), Vec::new()),
        Some(mut r) => {
            let back = get_decor_list(&mut r, &mut decor_budget)?;
            let front = get_decor_list(&mut r, &mut decor_budget)?;
            r.done("DECO")?;
            (back, front)
        }
    };

    let envelopes = match sections.get(*b"ENVL") {
        None => Vec::new(),
        Some(mut r) => {
            let list = get_envelopes(&mut r)?;
            r.done("ENVL")?;
            list
        }
    };

    let images = match sections.get(*b"IMGS") {
        None => Vec::new(),
        Some(mut r) => {
            let n = r.len(MAX_IMAGES, "Anzahl Bilder")?;
            let list = (0..n)
                .map(|_| {
                    Ok(Image {
                        name: r.str(MAX_NAME, "Bildname")?,
                        svg: r.bytes(MAX_IMAGE_BYTES, "Bild zu groß")?.to_vec(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            r.done("IMGS")?;
            list
        }
    };

    map.materials = materials;
    map.material_map = material_map;
    map.sky = sky;
    map.backgrounds = backgrounds;
    map.decor_back = decor_back;
    map.decor_front = decor_front;
    map.envelopes = envelopes;
    map.images = images;
    Ok(())
}
fn get_env_ref(r: &mut Reader<'_>) -> Result<Option<EnvRef>> {
    let index = r.u16()?;
    if index == u16::MAX {
        return Ok(None);
    }
    Ok(Some(EnvRef {
        index,
        offset_ms: r.i32()?,
    }))
}

fn get_decor_list(r: &mut Reader<'_>, budget: &mut usize) -> Result<Vec<Decor>> {
    let n = r.len(*budget, "zu viele Deko-Objekte")?;
    *budget -= n;
    let mut list = Vec::with_capacity(n);
    for _ in 0..n {
        let art = match r.u8()? {
            0 => Art::Builtin(r.str(MAX_NAME, "Grafikname")?),
            1 => Art::Image(r.u16()?),
            _ => return Err(MapError::Invalid("Grafik-Art")),
        };
        list.push(Decor {
            art,
            pos: r.vec2()?,
            scale: r.f32()?,
            rotation: r.f32()?,
            flip_x: r.bool()?,
            tint: r.rgba()?,
            pos_env: get_env_ref(r)?,
            color_env: get_env_ref(r)?,
        });
    }
    Ok(list)
}

fn get_envelopes(r: &mut Reader<'_>) -> Result<Vec<Envelope>> {
    let n = r.len(MAX_ENVELOPES, "Anzahl Animationen")?;
    let mut list = Vec::with_capacity(n);
    for _ in 0..n {
        let name = r.str(MAX_NAME, "Animationsname")?;
        let kind = match r.u8()? {
            0 => EnvKind::Position,
            1 => EnvKind::Color,
            _ => return Err(MapError::Invalid("Animations-Art")),
        };
        let synced = r.bool()?;
        let count = r.len(MAX_ENV_POINTS, "Anzahl Animationspunkte")?;
        let mut points: Vec<EnvPoint> = Vec::with_capacity(count);
        for _ in 0..count {
            let time_ms = r.u32()?;
            if points.last().is_some_and(|p| p.time_ms >= time_ms) {
                return Err(MapError::Invalid("Animationspunkte nicht aufsteigend"));
            }
            let value = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            let curve = lookup(&CURVES, r.u8()?, "Kurve")?;
            points.push(EnvPoint {
                time_ms,
                value,
                curve,
            });
        }
        list.push(Envelope {
            name,
            kind,
            synced,
            points,
        });
    }
    Ok(list)
}

/// Verweise auf Animationen und Bilder zeigen auf Vorhandenes der passenden Art.
fn check_references(map: &Map) -> Result<()> {
    let env_ok = |r: Option<EnvRef>, kind: EnvKind| {
        r.is_none_or(|r| {
            map.envelopes
                .get(usize::from(r.index))
                .is_some_and(|e| e.kind == kind)
        })
    };
    let all = map
        .backgrounds
        .iter()
        .flat_map(|b| &b.items)
        .chain(&map.decor_back)
        .chain(&map.decor_front);
    for d in all {
        if let Art::Image(i) = d.art
            && usize::from(i) >= map.images.len()
        {
            return Err(MapError::Invalid("Verweis auf fehlendes Bild"));
        }
        if !env_ok(d.pos_env, EnvKind::Position) || !env_ok(d.color_env, EnvKind::Color) {
            return Err(MapError::Invalid("Verweis auf fehlende Animation"));
        }
    }
    Ok(())
}

/// Spielbarkeit: mindestens ein Spawn, Flaggen nur paarweise.
///
/// # Errors
/// Ohne Spawn oder mit unpaarigen Flaggen.
pub fn validate(map: &Map) -> Result<()> {
    let count = |k| map.entities_of(k).count();
    let spawns =
        count(EntityKind::Spawn) + count(EntityKind::SpawnRed) + count(EntityKind::SpawnBlue);
    if spawns == 0 {
        return Err(MapError::NoSpawn);
    }
    let (red, blue) = (count(EntityKind::FlagRed), count(EntityKind::FlagBlue));
    if (red, blue) != (0, 0) && (red, blue) != (1, 1) {
        return Err(MapError::InvalidFlags { red, blue });
    }
    Ok(())
}

/// Prüfsumme einer Kartendatei (BLAKE2s-256 über die Datei-Bytes), identifiziert Karten
/// beim Download und im Zwischenspeicher (M6.5).
pub fn checksum(data: &[u8]) -> [u8; 32] {
    use blake2::Digest;
    blake2::Blake2s256::digest(data).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::Curve;

    fn rich_map() -> Map {
        let mut m = Map::from_rows("Test", &["#######", "#S.r.b#", "#=~!</#", "#######"]).unwrap();
        m.author = Some("Elora-Team".into());
        m.materials = vec!["earth".into(), "stone".into()];
        m.material_map = vec![0; m.tiles.len()];
        m.material_map[0] = 2;
        m.sky = Sky {
            top: Rgba::hex(0x112233),
            bottom: Rgba::hex(0x445566),
        };
        let mut bush = Decor::new(Art::Builtin("bush-1".into()), Vec2::new(10.0, 20.0));
        bush.pos_env = Some(EnvRef {
            index: 0,
            offset_ms: -250,
        });
        bush.color_env = Some(EnvRef {
            index: 1,
            offset_ms: 0,
        });
        let mut own = Decor::new(Art::Image(0), Vec2::new(-5.0, 3.5));
        own.scale = 2.0;
        own.rotation = 45.0;
        own.flip_x = true;
        m.decor_back = vec![bush.clone()];
        m.decor_front = vec![own];
        m.backgrounds = vec![Background {
            name: "Wolken".into(),
            parallax: Vec2::new(0.2, 0.1),
            offset: Vec2::new(0.0, -100.0),
            repeat_x: Some(800.0),
            items: vec![bush],
        }];
        let point = |time_ms, v| EnvPoint {
            time_ms,
            value: [v, 0.0, 0.0, 1.0],
            curve: Curve::Smooth,
        };
        m.envelopes = vec![
            Envelope {
                name: "Wippen".into(),
                kind: EnvKind::Position,
                synced: true,
                points: vec![point(0, 0.0), point(500, 4.0), point(1000, 0.0)],
            },
            Envelope {
                name: "Blinken".into(),
                kind: EnvKind::Color,
                synced: false,
                points: vec![point(0, 1.0), point(300, 0.5)],
            },
        ];
        m.images = vec![Image {
            name: "eigen".into(),
            svg: b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec(),
        }];
        m
    }

    #[test]
    fn roundtrip_keeps_everything() {
        let m = rich_map();
        let data = encode(&m);
        assert_eq!(&data[..4], b"EMAP");
        assert_eq!(decode(&data).unwrap(), m);
        // Nur Kollision: optionale Abschnitte fehlen, Standardwerte kommen zurück
        let plain = Map::from_rows("P", &["###", "#S#", "###"]).unwrap();
        assert_eq!(decode(&encode(&plain)).unwrap(), plain);
        // Gleiche Karte → gleiche Bytes → gleiche Prüfsumme
        assert_eq!(checksum(&encode(&m)), checksum(&data));
        assert_ne!(checksum(&encode(&plain)), checksum(&data));
    }

    #[test]
    fn rejects_foreign_and_damaged_files() {
        let data = encode(&rich_map());
        assert_eq!(decode(b"").unwrap_err(), MapError::BadMagic);
        assert_eq!(decode(b"format = 1").unwrap_err(), MapError::BadMagic);
        let mut v = data.clone();
        v[4] = 9;
        assert_eq!(
            decode(&v).unwrap_err(),
            MapError::UnsupportedFormat { found: 9 }
        );
        // jede Kürzung und jedes gekippte Byte führt zu einem Fehler, nie zu einer Panik
        for cut in 0..data.len() {
            assert!(decode(&data[..cut]).is_err(), "gekürzt auf {cut}");
        }
        for i in 6..data.len() {
            let mut v = data.clone();
            v[i] ^= 0x55;
            let _ = decode(&v);
        }
    }

    /// Kodiert eigene Abschnitte, um einzelne Prüfungen gezielt auszulösen.
    fn raw(sections: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut w = Writer::default();
        for (tag, body) in sections {
            w.section(**tag, &Writer(body.clone()));
        }
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(&w.0, 1));
        out
    }

    fn info() -> (&'static [u8; 4], Vec<u8>) {
        let mut w = Writer::default();
        w.str("x");
        w.str("");
        (b"INFO", w.0)
    }

    fn game(width: u32, height: u32, tiles: &[u8]) -> (&'static [u8; 4], Vec<u8>) {
        let mut w = Writer::default();
        w.u32(width);
        w.u32(height);
        w.0.extend_from_slice(tiles);
        (b"GAME", w.0)
    }

    fn ents(list: &[(u8, u32, u32)]) -> (&'static [u8; 4], Vec<u8>) {
        let mut w = Writer::default();
        w.len(list.len());
        for &(k, x, y) in list {
            w.u8(k);
            w.u32(x);
            w.u32(y);
        }
        (b"ENTS", w.0)
    }

    #[test]
    fn checks_sections_and_limits() {
        let ok = [info(), game(1, 1, &[0]), ents(&[(0, 0, 0)])];
        assert!(decode(&raw(&ok)).is_ok());
        assert_eq!(
            decode(&raw(&ok[..2])).unwrap_err(),
            MapError::MissingSection("ENTS")
        );
        assert!(matches!(
            decode(&raw(&[
                info(),
                info(),
                game(1, 1, &[0]),
                ents(&[(0, 0, 0)])
            ])),
            Err(MapError::DuplicateSection(_))
        ));
        // unbekannte Abschnitte werden übersprungen
        assert!(
            decode(&raw(&[
                ok[0].clone(),
                (b"NEU!", vec![1, 2, 3]),
                ok[1].clone(),
                ok[2].clone()
            ]))
            .is_ok()
        );
        assert_eq!(
            decode(&raw(&[info(), game(1001, 1, &[]), ents(&[])])).unwrap_err(),
            MapError::TooLarge {
                width: 1001,
                height: 1
            }
        );
        assert_eq!(
            decode(&raw(&[info(), game(1, 1, &[99]), ents(&[(0, 0, 0)])])).unwrap_err(),
            MapError::Invalid("Tile-Art")
        );
        assert_eq!(
            decode(&raw(&[info(), game(1, 1, &[0]), ents(&[(0, 1, 0)])])).unwrap_err(),
            MapError::Invalid("Entity außerhalb")
        );
        assert_eq!(
            decode(&raw(&[info(), game(1, 1, &[0]), ents(&[])])).unwrap_err(),
            MapError::NoSpawn
        );
        assert!(decode_draft(&raw(&[info(), game(1, 1, &[0]), ents(&[])])).is_ok());
        assert_eq!(
            decode(&raw(&[
                info(),
                game(2, 1, &[0, 0]),
                ents(&[(0, 0, 0), (3, 1, 0)])
            ]))
            .unwrap_err(),
            MapError::InvalidFlags { red: 1, blue: 0 }
        );
    }

    #[test]
    fn rejects_zip_bomb_and_bad_references() {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
            &vec![0; MAX_PAYLOAD + 1],
            9,
        ));
        assert!(out.len() < 100_000);
        assert_eq!(decode(&out).unwrap_err(), MapError::PayloadTooLarge);

        let mut m = rich_map();
        m.decor_front[0].art = Art::Image(5);
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("Verweis auf fehlendes Bild")
        );
        let mut m = rich_map();
        // Farb-Animation als Bewegung benutzt
        m.decor_back[0].pos_env = Some(EnvRef {
            index: 1,
            offset_ms: 0,
        });
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("Verweis auf fehlende Animation")
        );
        let mut m = rich_map();
        m.images[0].svg = vec![b' '; MAX_IMAGE_BYTES + 1];
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("Bild zu groß")
        );
        let mut m = rich_map();
        m.envelopes[0].points[1].time_ms = 0;
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("Animationspunkte nicht aufsteigend")
        );
        let mut m = rich_map();
        m.material_map[3] = 3;
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("Materialraster")
        );
    }
}
