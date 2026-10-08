//! Release map format `.emap` (E-129, E-143 to E-146), described in
//! `docs/handbook/map-format.md`.
//!
//! File: `EMAP` + format version (u16) + zlib-compressed sections. Each section is
//! `identifier (4 bytes) | length (u32) | content`. Numbers are
//! little endian.

use elora_sim::{BeltDir, DummyPattern, JumpDir, Tile, Vec2};

use crate::adventure::{
    Adventure, CameraMode, MAX_LIST, MAX_OBJECTS, Object, ObjectKind, SwitchTrigger,
};
use crate::look::{
    Art, Background, Curve, Decor, EnvKind, EnvPoint, EnvRef, Envelope, Image, Rgba, Sky, Weather,
    WeatherKind,
};
use crate::{Entity, EntityKind, MAX_SIZE, Map};

/// Identifier at the start of the file.
pub const MAGIC: [u8; 4] = *b"EMAP";
/// Supported version of the binary format.
pub const FORMAT_VERSION: u16 = 1;

/// At most this many bytes are decompressed (protection against zip bombs).
pub const MAX_PAYLOAD: usize = 32 << 20;
/// Size of an embedded SVG.
pub const MAX_IMAGE_BYTES: usize = 512 << 10;
pub const MAX_IMAGES: usize = 64;
pub const MAX_MATERIALS: usize = 255;
pub const MAX_BACKGROUNDS: usize = 16;
/// Decoration objects in total (all layers).
pub const MAX_DECOR: usize = 20_000;
pub const MAX_ENVELOPES: usize = 256;
pub const MAX_ENV_POINTS: usize = 1024;
/// Length of names and identifiers in bytes.
pub const MAX_NAME: usize = 128;
/// Length of conditions in the adventure section.
pub const MAX_CONDITION: usize = 1024;

/// Error while reading a map.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MapError {
    #[error("not an Elora map (magic missing)")]
    BadMagic,
    #[error("unsupported format version {found} (supported: {FORMAT_VERSION})")]
    UnsupportedFormat { found: u16 },
    #[error("map data is corrupt (decompression failed)")]
    Compression,
    #[error("map data is too large (over {MAX_PAYLOAD} bytes unpacked)")]
    PayloadTooLarge,
    #[error("map data ends too early")]
    Truncated,
    #[error("section `{0}` missing")]
    MissingSection(&'static str),
    #[error("section `{0}` appears twice")]
    DuplicateSection(String),
    #[error("invalid map data: {0}")]
    Invalid(&'static str),
    #[error("grid {width}×{height} is larger than allowed ({MAX_SIZE}×{MAX_SIZE})")]
    TooLarge { width: usize, height: usize },
    #[error("line {line}: length {found}, expected {expected}")]
    RaggedRow {
        line: usize,
        found: usize,
        expected: usize,
    },
    #[error("line {line}, column {column}: unknown character `{symbol}`")]
    UnknownSymbol {
        line: usize,
        column: usize,
        symbol: char,
    },
    #[error("adventure objects: {0}")]
    Adventure(String),
    #[error("map has no spawn point")]
    NoSpawn,
    #[error("flags: {red}× red, {blue}× blue – exactly one each for CTF, otherwise none")]
    InvalidFlags { red: usize, blue: usize },
}

type Result<T> = std::result::Result<T, MapError>;

// ---------------------------------------------------------------- Encoding of the enums

const TILES: [Tile; 17] = [
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
    Tile::Climb,
    Tile::Crumble,
    Tile::HookPoint,
    Tile::Quicksand,
    Tile::ThinIce,
    Tile::IceWater,
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
    let i = table.iter().position(|t| t == v).expect("table complete");
    u8::try_from(i).expect("table < 256")
}

fn lookup<T: Copy>(table: &[T], c: u8, what: &'static str) -> Result<T> {
    table
        .get(usize::from(c))
        .copied()
        .ok_or(MapError::Invalid(what))
}

// ---------------------------------------------------------------- Writing

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
        self.u32(u32::try_from(n).expect("length fits in u32"));
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

/// Writes a map in the binary format.
///
/// # Panics
/// If the map exceeds the limits of the format (e.g. over 4 GB) – [`decode`] rejects such
/// maps.
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

    // Weather (R2-W1): only if there is any – old programs skip the section
    if !map.weather.is_clear() {
        let mut s = Writer::default();
        s.u8(code(&WeatherKind::ALL, &map.weather.kind));
        s.f32(map.weather.intensity);
        s.f32(map.weather.wind);
        payload.section(*b"WTHR", &s);
    }

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

    if !map.adventure.objects.is_empty() {
        let mut s = Writer::default();
        put_adventure(&mut s, &map.adventure);
        payload.section(*b"ADVN", &s);
    }

    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(&payload.0, 9));
    out
}

fn put_adventure(w: &mut Writer, a: &Adventure) {
    w.len(a.objects.len());
    for o in &a.objects {
        w.str(&o.id);
        w.vec2(o.pos);
        match &o.kind {
            ObjectKind::Creature { kind, persistent } => {
                w.u8(0);
                w.str(kind);
                w.u8(u8::from(*persistent));
            }
            ObjectKind::Npc {
                character,
                dialog,
                facing,
                walk,
            } => {
                w.u8(1);
                w.str(character);
                w.str(dialog);
                w.u8(u8::from(*facing > 0));
                w.f32(*walk);
            }
            ObjectKind::Chest { contents, lock } => {
                w.u8(2);
                w.len(contents.len());
                for (item, n) in contents {
                    w.str(item);
                    w.u32(*n);
                }
                w.str(lock);
            }
            ObjectKind::Switch {
                flag,
                once,
                trigger,
            } => {
                w.u8(3);
                w.str(flag);
                w.u8(u8::from(*once));
                w.u8(match trigger {
                    SwitchTrigger::Interact => 0,
                    SwitchTrigger::Hammer => 1,
                    SwitchTrigger::Hook => 2,
                });
            }
            ObjectKind::Door { size, open_if } => {
                w.u8(4);
                w.u8(size.0);
                w.u8(size.1);
                w.str(open_if);
            }
            ObjectKind::Collectible { item } => {
                w.u8(5);
                w.str(item);
            }
            ObjectKind::SavePoint => w.u8(6),
            ObjectKind::HealPlant { heal } => {
                w.u8(7);
                w.i32(*heal);
            }
            ObjectKind::Spawn => w.u8(8),
            ObjectKind::Exit {
                size,
                map,
                spawn,
                on_touch,
            } => {
                w.u8(9);
                w.vec2(*size);
                w.str(map);
                w.str(spawn);
                w.u8(u8::from(*on_touch));
            }
            ObjectKind::Zone { size } => {
                w.u8(10);
                w.vec2(*size);
            }
            ObjectKind::Camera { size, mode } => {
                w.u8(11);
                w.vec2(*size);
                w.u8(match mode {
                    CameraMode::Fixed => 0,
                    CameraMode::Bounds => 1,
                });
            }
        }
    }
}

fn get_adventure(r: &mut Reader<'_>) -> Result<Adventure> {
    let n = r.len(MAX_OBJECTS, "number of adventure objects")?;
    let mut objects = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.str(MAX_NAME, "object id")?;
        let pos = r.vec2()?;
        let kind = match r.u8()? {
            0 => ObjectKind::Creature {
                kind: r.str(MAX_NAME, "creature kind")?,
                persistent: r.bool()?,
            },
            1 => ObjectKind::Npc {
                character: r.str(MAX_NAME, "character")?,
                dialog: r.str(MAX_NAME, "dialog")?,
                facing: if r.bool()? { 1 } else { -1 },
                walk: r.f32()?,
            },
            2 => {
                let k = r.len(MAX_LIST, "chest contents")?;
                let contents = (0..k)
                    .map(|_| Ok((r.str(MAX_NAME, "item")?, r.u32()?)))
                    .collect::<Result<Vec<_>>>()?;
                ObjectKind::Chest {
                    contents,
                    lock: r.str(MAX_CONDITION, "lock")?,
                }
            }
            3 => ObjectKind::Switch {
                flag: r.str(MAX_NAME, "flag")?,
                once: r.bool()?,
                trigger: match r.u8()? {
                    0 => SwitchTrigger::Interact,
                    1 => SwitchTrigger::Hammer,
                    2 => SwitchTrigger::Hook,
                    _ => return Err(MapError::Invalid("switch trigger")),
                },
            },
            4 => ObjectKind::Door {
                size: (r.u8()?, r.u8()?),
                open_if: r.str(MAX_CONDITION, "door condition")?,
            },
            5 => ObjectKind::Collectible {
                item: r.str(MAX_NAME, "item")?,
            },
            6 => ObjectKind::SavePoint,
            7 => ObjectKind::HealPlant { heal: r.i32()? },
            8 => ObjectKind::Spawn,
            9 => ObjectKind::Exit {
                size: r.vec2()?,
                map: r.str(MAX_NAME, "target map")?,
                spawn: r.str(MAX_NAME, "target entrance")?,
                on_touch: r.bool()?,
            },
            10 => ObjectKind::Zone { size: r.vec2()? },
            11 => ObjectKind::Camera {
                size: r.vec2()?,
                mode: match r.u8()? {
                    0 => CameraMode::Fixed,
                    1 => CameraMode::Bounds,
                    _ => return Err(MapError::Invalid("camera kind")),
                },
            },
            _ => return Err(MapError::Invalid("adventure object kind")),
        };
        objects.push(Object { id, pos, kind });
    }
    Ok(Adventure { objects })
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

// ---------------------------------------------------------------- Reading

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
        Ok(self.take(N)?.try_into().expect("length matches"))
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(MapError::Invalid("boolean")),
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
            Err(MapError::Invalid("number is not finite"))
        }
    }
    fn vec2(&mut self) -> Result<Vec2> {
        Ok(Vec2::new(self.f32()?, self.f32()?))
    }
    /// Count/length with an upper limit.
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

/// The sections of a file, by identifier.
#[derive(Default)]
struct Sections<'a> {
    found: Vec<([u8; 4], &'a [u8])>,
}

impl<'a> Sections<'a> {
    fn parse(mut r: Reader<'a>) -> Result<Self> {
        let mut s = Self::default();
        while !r.0.is_empty() {
            let tag = r.array::<4>()?;
            let body = r.bytes(MAX_PAYLOAD, "section length")?;
            if s.found.iter().any(|(t, _)| *t == tag) {
                return Err(MapError::DuplicateSection(
                    String::from_utf8_lossy(&tag).into_owned(),
                ));
            }
            // Unknown sections are left alone (later extensions of the same version)
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

    /// `&'static`, so that the identifier appears in the error.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn require(&self, tag: &'static [u8; 4]) -> Result<Reader<'a>> {
        self.get(*tag).ok_or(MapError::MissingSection(
            std::str::from_utf8(tag).expect("tag is ASCII"),
        ))
    }
}

/// Reads a map from the binary format and checks it.
///
/// # Errors
/// On a wrong identifier or version, corrupt data, exceeded limits or an
/// unplayable map (no spawn, wrong flags).
pub fn decode(data: &[u8]) -> Result<Map> {
    let map = decode_draft(data)?;
    validate(&map)?;
    Ok(map)
}

/// Like [`decode`], but without the playability check (spawn, flags) – for drafts in the
/// editor.
/// All protection limits and references are still checked.
///
/// # Errors
/// On a wrong identifier or version, corrupt data or exceeded limits.
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
    let name = r.str(MAX_NAME, "map name")?;
    let author = r.str(MAX_NAME, "author")?;
    r.done("INFO")?;

    let mut r = sections.require(b"GAME")?;
    let width = r.len(usize::MAX, "width")?;
    let height = r.len(usize::MAX, "height")?;
    if width == 0 || height == 0 {
        return Err(MapError::Invalid("empty grid"));
    }
    if width > MAX_SIZE || height > MAX_SIZE {
        return Err(MapError::TooLarge { width, height });
    }
    let tiles = r
        .take(width * height)?
        .iter()
        .map(|&c| lookup(&TILES, c, "tile kind"))
        .collect::<Result<Vec<_>>>()?;
    r.done("GAME")?;

    let mut r = sections.require(b"ENTS")?;
    let n = r.len(width * height, "number of entities")?;
    let mut entities = Vec::with_capacity(n);
    for _ in 0..n {
        let kind = lookup(&ENTITIES, r.u8()?, "entity kind")?;
        let tx = r.len(width - 1, "entity out of bounds")?;
        let ty = r.len(height - 1, "entity out of bounds")?;
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
    if let Some(mut r) = sections.get(*b"ADVN") {
        map.adventure = get_adventure(&mut r)?;
        r.done("ADVN")?;
        map.adventure
            .validate(map.width, map.height)
            .map_err(MapError::Adventure)?;
    }
    check_references(&map)?;
    Ok(map)
}

/// Optional sections for the look (materials, sky, layers, decoration, animations, images).
fn decode_look(sections: &Sections<'_>, map: &mut Map) -> Result<()> {
    let (width, height) = (map.width, map.height);
    let (materials, material_map) = match sections.get(*b"MATL") {
        None => (Vec::new(), Vec::new()),
        Some(mut r) => {
            let n = r.len(MAX_MATERIALS, "number of materials")?;
            let materials = (0..n)
                .map(|_| r.str(MAX_NAME, "material"))
                .collect::<Result<Vec<_>>>()?;
            let map = r.bytes(width * height, "material grid")?.to_vec();
            if map.len() != width * height || map.iter().any(|&m| usize::from(m) > n) {
                return Err(MapError::Invalid("material grid"));
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

    let weather = sections
        .get(*b"WTHR")
        .map_or(Ok(Weather::CLEAR), get_weather)?;

    let mut decor_budget = MAX_DECOR;
    let backgrounds = match sections.get(*b"BGRD") {
        None => Vec::new(),
        Some(mut r) => {
            let n = r.len(MAX_BACKGROUNDS, "number of background layers")?;
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                let name = r.str(MAX_NAME, "layer name")?;
                let parallax = r.vec2()?;
                let offset = r.vec2()?;
                let repeat = r.f32()?;
                if repeat < 0.0 {
                    return Err(MapError::Invalid("repeat"));
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
            let n = r.len(MAX_IMAGES, "number of images")?;
            let list = (0..n)
                .map(|_| {
                    Ok(Image {
                        name: r.str(MAX_NAME, "image name")?,
                        svg: r.bytes(MAX_IMAGE_BYTES, "image too large")?.to_vec(),
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
    map.weather = weather;
    map.backgrounds = backgrounds;
    map.decor_back = decor_back;
    map.decor_front = decor_front;
    map.envelopes = envelopes;
    map.images = images;
    Ok(())
}
/// Section `WTHR` (R2-W1): kind, strength 0–1, wind −1–1.
fn get_weather(mut r: Reader<'_>) -> Result<Weather> {
    let kind = *WeatherKind::ALL
        .get(usize::from(r.u8()?))
        .ok_or(MapError::Invalid("weather kind"))?;
    let (intensity, wind) = (r.f32()?, r.f32()?);
    if !(0.0..=1.0).contains(&intensity) || !(-1.0..=1.0).contains(&wind) {
        return Err(MapError::Invalid("weather values"));
    }
    r.done("WTHR")?;
    Ok(Weather {
        kind,
        intensity,
        wind,
    })
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
    let n = r.len(*budget, "too many decor objects")?;
    *budget -= n;
    let mut list = Vec::with_capacity(n);
    for _ in 0..n {
        let art = match r.u8()? {
            0 => Art::Builtin(r.str(MAX_NAME, "art name")?),
            1 => Art::Image(r.u16()?),
            _ => return Err(MapError::Invalid("art kind")),
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
    let n = r.len(MAX_ENVELOPES, "number of animations")?;
    let mut list = Vec::with_capacity(n);
    for _ in 0..n {
        let name = r.str(MAX_NAME, "animation name")?;
        let kind = match r.u8()? {
            0 => EnvKind::Position,
            1 => EnvKind::Color,
            _ => return Err(MapError::Invalid("animation kind")),
        };
        let synced = r.bool()?;
        let count = r.len(MAX_ENV_POINTS, "number of animation points")?;
        let mut points: Vec<EnvPoint> = Vec::with_capacity(count);
        for _ in 0..count {
            let time_ms = r.u32()?;
            if points.last().is_some_and(|p| p.time_ms >= time_ms) {
                return Err(MapError::Invalid("animation points not ascending"));
            }
            let value = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            let curve = lookup(&CURVES, r.u8()?, "curve")?;
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

/// References to animations and images point to existing entries of the matching kind.
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
            return Err(MapError::Invalid("reference to missing image"));
        }
        if !env_ok(d.pos_env, EnvKind::Position) || !env_ok(d.color_env, EnvKind::Color) {
            return Err(MapError::Invalid("reference to missing animation"));
        }
    }
    Ok(())
}

/// Playability: at least one spawn, flags only in pairs.
///
/// # Errors
/// Without a spawn or with unpaired flags.
pub fn validate(map: &Map) -> Result<()> {
    let count = |k| map.entities_of(k).count();
    let spawns =
        count(EntityKind::Spawn) + count(EntityKind::SpawnRed) + count(EntityKind::SpawnBlue);
    // Adventure maps may only have entrances (A1.5)
    let entrances = map
        .adventure
        .objects
        .iter()
        .filter(|o| matches!(o.kind, ObjectKind::Spawn))
        .count();
    if spawns + entrances == 0 {
        return Err(MapError::NoSpawn);
    }
    let (red, blue) = (count(EntityKind::FlagRed), count(EntityKind::FlagBlue));
    if (red, blue) != (0, 0) && (red, blue) != (1, 1) {
        return Err(MapError::InvalidFlags { red, blue });
    }
    Ok(())
}

/// Checksum of a map file (BLAKE2s-256 over the file bytes), identifies maps
/// during download and in the cache (M6.5).
pub fn checksum(data: &[u8]) -> [u8; 32] {
    use blake2::Digest;
    blake2::Blake2s256::digest(data).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::Curve;

    /// All kinds of adventure objects.
    #[allow(clippy::too_many_lines)] // a list of all object kinds
    fn adventure_map() -> Map {
        let mut m = Map::from_rows(
            "Wiese",
            &["##########", "#S.......#", "#........#", "##########"],
        )
        .unwrap();
        let o = |id: &str, x: f32, y: f32, kind| Object {
            id: id.into(),
            pos: Vec2::new(x, y),
            kind,
        };
        m.adventure.objects = vec![
            o("eingang", 48.0, 80.0, ObjectKind::Spawn),
            o(
                "kaefer-1",
                100.0,
                80.0,
                ObjectKind::Creature {
                    kind: "stachelkaefer".into(),
                    persistent: false,
                },
            ),
            o(
                "hummel",
                140.0,
                60.0,
                ObjectKind::Creature {
                    kind: "hummel".into(),
                    persistent: true,
                },
            ),
            o(
                "oma",
                60.0,
                80.0,
                ObjectKind::Npc {
                    character: "oma".into(),
                    dialog: "oma".into(),
                    facing: -1,
                    walk: 32.0,
                },
            ),
            o(
                "truhe-1",
                200.0,
                80.0,
                ObjectKind::Chest {
                    contents: vec![("glanztropfen".into(), 20), ("tauumhang".into(), 1)],
                    lock: "hat schluessel".into(),
                },
            ),
            o(
                "hebel",
                220.0,
                80.0,
                ObjectKind::Switch {
                    flag: "tor.wiese".into(),
                    once: false,
                    trigger: SwitchTrigger::Hammer,
                },
            ),
            o(
                "tor",
                256.0,
                32.0,
                ObjectKind::Door {
                    size: (1, 2),
                    open_if: "merker tor.wiese".into(),
                },
            ),
            o(
                "stein",
                120.0,
                70.0,
                ObjectKind::Collectible {
                    item: "glitzerstein".into(),
                },
            ),
            o("quellstein", 160.0, 80.0, ObjectKind::SavePoint),
            o("blume", 180.0, 84.0, ObjectKind::HealPlant { heal: 2 }),
            o(
                "weg-ost",
                288.0,
                32.0,
                ObjectKind::Exit {
                    size: Vec2::new(32.0, 64.0),
                    map: "wiese-2".into(),
                    spawn: "west".into(),
                    on_touch: true,
                },
            ),
            o(
                "bruecke",
                64.0,
                32.0,
                ObjectKind::Zone {
                    size: Vec2::new(64.0, 64.0),
                },
            ),
            o(
                "arena",
                32.0,
                32.0,
                ObjectKind::Camera {
                    size: Vec2::new(256.0, 64.0),
                    mode: CameraMode::Bounds,
                },
            ),
        ];
        m
    }

    #[test]
    fn adventure_objects_roundtrip() {
        let m = adventure_map();
        let back = decode(&encode(&m)).unwrap();
        assert_eq!(back.adventure, m.adventure);
        // no objects, no section: multiplayer maps stay identical byte for byte
        let mut plain = m.clone();
        plain.adventure = Adventure::default();
        assert!(!encode(&plain).is_empty());
        assert_eq!(
            decode(&encode(&plain)).unwrap().adventure,
            Adventure::default()
        );
    }

    #[test]
    fn adventure_map_needs_no_multiplayer_spawn() {
        let mut m = adventure_map();
        m.entities.clear();
        assert!(decode(&encode(&m)).is_ok(), "entrance is enough");
        m.adventure
            .objects
            .retain(|o| !matches!(o.kind, ObjectKind::Spawn));
        assert_eq!(decode(&encode(&m)), Err(MapError::NoSpawn));
    }

    #[test]
    fn broken_adventure_objects_are_rejected() {
        type Breaker = fn(&mut Map);
        let cases: [(Breaker, &str); 5] = [
            (
                |m| m.adventure.objects[1].id = "eingang".into(),
                "duplicate id",
            ),
            (
                |m| m.adventure.objects[1].pos = Vec2::new(5000.0, 0.0),
                "outside",
            ),
            (
                |m| m.adventure.objects[6].pos = Vec2::new(250.0, 32.0),
                "grid",
            ),
            (
                |m| {
                    if let ObjectKind::Exit { spawn, .. } = &mut m.adventure.objects[10].kind {
                        spawn.clear();
                    }
                },
                "target missing",
            ),
            (
                |m| {
                    if let ObjectKind::Zone { size } = &mut m.adventure.objects[11].kind {
                        *size = Vec2::new(0.0, 10.0);
                    }
                },
                "area",
            ),
        ];
        for (break_it, expected) in cases {
            let mut m = adventure_map();
            break_it(&mut m);
            let e = decode(&encode(&m)).unwrap_err().to_string();
            assert!(e.contains(expected), "{expected}: {e}");
        }
        // truncated section
        let mut w = Writer::default();
        put_adventure(&mut w, &adventure_map().adventure);
        let cut = &w.0[..w.0.len() - 3];
        assert_eq!(
            get_adventure(&mut Reader(cut)).unwrap_err(),
            MapError::Truncated
        );
        let mut bad = w.0.clone();
        bad[4 + 4 + 7 + 8] = 99; // kind of the first object
        assert!(get_adventure(&mut Reader(&bad)).is_err());
    }

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
        // Collision only: optional sections are missing, default values come back
        let plain = Map::from_rows("P", &["###", "#S#", "###"]).unwrap();
        assert_eq!(decode(&encode(&plain)).unwrap(), plain);
        // Same map → same bytes → same checksum
        assert_eq!(checksum(&encode(&m)), checksum(&data));
        assert_ne!(checksum(&encode(&plain)), checksum(&data));
    }

    /// Weather (R2-W1): round trip; without weather no section (old maps stay the same).
    #[test]
    fn weather_roundtrip_and_absent_section() {
        let plain = Map::from_rows("P", &["###", "#S#", "###"]).unwrap();
        let without = encode(&plain);
        for kind in WeatherKind::ALL {
            let mut m = plain.clone();
            m.weather = Weather {
                kind,
                intensity: 0.6,
                wind: -0.4,
            };
            let back = decode(&encode(&m)).unwrap();
            assert_eq!(
                back.weather,
                if kind == WeatherKind::Clear {
                    Weather::CLEAR
                } else {
                    m.weather
                }
            );
        }
        // clear map: same bytes as before weather
        let mut clear = plain.clone();
        clear.weather = Weather::CLEAR;
        assert_eq!(encode(&clear), without);
    }

    #[test]
    fn rejects_bad_weather() {
        let base = [info(), game(1, 1, &[0]), ents(&[(0, 0, 0)])];
        let weather = |kind: u8, intensity: f32, wind: f32| {
            let mut w = Writer::default();
            w.u8(kind);
            w.f32(intensity);
            w.f32(wind);
            (b"WTHR", w.0)
        };
        let mut ok = base.to_vec();
        ok.push(weather(1, 0.5, 0.0));
        assert_eq!(decode(&raw(&ok)).unwrap().weather.kind, WeatherKind::Rain);
        for bad in [
            weather(99, 0.5, 0.0),
            weather(1, 2.0, 0.0),
            weather(1, 0.5, -3.0),
            weather(1, f32::NAN, 0.0),
        ] {
            let mut list = base.to_vec();
            list.push(bad);
            assert!(decode(&raw(&list)).is_err());
        }
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
        // every truncation and every flipped byte leads to an error, never to a panic
        for cut in 0..data.len() {
            assert!(decode(&data[..cut]).is_err(), "truncated to {cut}");
        }
        for i in 6..data.len() {
            let mut v = data.clone();
            v[i] ^= 0x55;
            let _ = decode(&v);
        }
    }

    /// Encodes custom sections to trigger individual checks on purpose.
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
        // unknown sections are skipped
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
            MapError::Invalid("tile kind")
        );
        assert_eq!(
            decode(&raw(&[info(), game(1, 1, &[0]), ents(&[(0, 1, 0)])])).unwrap_err(),
            MapError::Invalid("entity out of bounds")
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
            MapError::Invalid("reference to missing image")
        );
        let mut m = rich_map();
        // color animation used as movement
        m.decor_back[0].pos_env = Some(EnvRef {
            index: 1,
            offset_ms: 0,
        });
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("reference to missing animation")
        );
        let mut m = rich_map();
        m.images[0].svg = vec![b' '; MAX_IMAGE_BYTES + 1];
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("image too large")
        );
        let mut m = rich_map();
        m.envelopes[0].points[1].time_ms = 0;
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("animation points not ascending")
        );
        let mut m = rich_map();
        m.material_map[3] = 3;
        assert_eq!(
            decode(&encode(&m)).unwrap_err(),
            MapError::Invalid("material grid")
        );
    }
}
