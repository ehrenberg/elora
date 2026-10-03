//! Abenteuer-Menü (A1.7, E-225, E-263, E-264): Inventar, Fähigkeiten, Aufgaben, Weltkarte –
//! sowie Lottes Laden und Klonks Schmiede, die Gespräche öffnen. Das Spiel steht solange.
//!
//! Zeichnet in Bildschirm-Pixeln über [`crate::ui`] und meldet [`Command`]s; ausgeführt
//! werden sie in `app_adventure.rs` auf dem Spielstand.

// Layout-Code: `s` (Skalierung), `w`/`h`/`x`/`y` wie in menu.rs
#![allow(clippy::many_single_char_names, clippy::too_many_lines)]

use elora_adventure::data::{Bonus, Branch, ItemKind, Rarity, Slot};
use elora_adventure::quest::{QuestKind, QuestStatus};
use elora_adventure::state::Refusal;
use elora_adventure::{Content, GLANZTROPFEN, SaveGame};
use elora_render::{Align, Color, Tint};
use elora_sim::{Vec2, Weapon};

use crate::creatures::CreatureArt;
use crate::figure::FigureArt;
use crate::lang::Lang;
use crate::ui::{self, Rect, Ui};

/// Was das Menü zeigt.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Inventory,
    Skills,
    Quests,
    Map,
    /// Laden einer Figur (Id aus `shops.toml`).
    Shop(String),
    Forge,
}

/// Auswahl innerhalb der Seiten (bleibt beim Wechseln erhalten).
#[derive(Debug, Clone, Default)]
pub struct MenuState {
    pub panel: Panel,
    pub item: Option<String>,
    pub quest: Option<String>,
    pub selling: bool,
    /// Letzte Ablehnung zum Anzeigen.
    pub refusal: Option<Refusal>,
}

/// Was die App ausführen soll.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Close,
    Equip(String),
    Unequip(Slot),
    Use(String),
    Learn(String),
    Buy(String, String),
    Sell(String),
    Upgrade(Weapon),
}

/// Daten für einen Frame.
pub struct MenuData<'a> {
    pub lang: &'a Lang,
    pub code: &'a str,
    pub content: &'a Content,
    pub save: &'a SaveGame,
    pub art: &'a CreatureArt,
    pub figure: &'a FigureArt,
    pub tint: &'a Tint,
    pub screen: Vec2,
}

const TAB_COLORS: [Color; 4] = [ui::ORANGE, ui::GREEN, ui::BLUE, ui::VIOLET];
const PRICE_BAD: Color = Color::hex(0xd94a4a);
const LOCKED: Color = Color::hex(0xd8ccb8);

pub fn refusal_key(r: Refusal) -> &'static str {
    match r {
        Refusal::Unknown => "refusal.unknown",
        Refusal::NoPoints => "refusal.no_points",
        Refusal::MaxRank => "refusal.max_rank",
        Refusal::Locked => "refusal.locked",
        Refusal::NeedsAbility => "refusal.needs_ability",
        Refusal::TooExpensive => "refusal.too_expensive",
        Refusal::MissingMaterial => "refusal.missing_material",
        Refusal::NotOwned => "refusal.not_owned",
        Refusal::WrongKind => "refusal.wrong_kind",
        Refusal::Full => "refusal.full",
        Refusal::NoWeapon => "refusal.no_weapon",
        Refusal::MaxLevel => "refusal.max_level",
    }
}

/// Text eines Bonus („+1 Leben“).
pub fn bonus_text(lang: &Lang, b: Bonus) -> String {
    let (key, n): (&str, String) = match b {
        Bonus::MaxHealth(v) => ("bonus.max_health", format!("{v:+}")),
        Bonus::Armor(v) => ("bonus.armor", format!("{v:+}")),
        Bonus::DamagePct(v) => ("bonus.damage_pct", format!("{v:+}")),
        Bonus::FireDelayPct(v) => ("bonus.fire_delay_pct", format!("{v:+}")),
        Bonus::KnockbackPct(v) => ("bonus.knockback_pct", format!("{v:+}")),
        Bonus::Ammo(v) => ("bonus.ammo", format!("{v:+}")),
        Bonus::Magnet(v) => ("bonus.magnet", format!("{v:+}")),
        Bonus::DropsPct(v) => ("bonus.drops_pct", format!("{v:+}")),
        Bonus::InvulnerableMs(v) => ("bonus.invulnerable_ms", format!("{v:+}")),
        Bonus::HealBonus(v) => ("bonus.heal_bonus", format!("{v:+}")),
        Bonus::SecondChance(v) => ("bonus.second_chance", v.to_string()),
        Bonus::RuckCooldownMs(v) => ("bonus.ruck_cooldown_ms", format!("{v:+}")),
        Bonus::HookLengthPct(v) => ("bonus.hook_length_pct", format!("{v:+}")),
        Bonus::StompRadius(v) => ("bonus.stomp_radius", format!("{v:+}")),
        Bonus::StompDamage(v) => ("bonus.stomp_damage", format!("{v:+}")),
        Bonus::GripMs(v) => ("bonus.grip_ms", format!("{v:+}")),
        Bonus::GlideFall(v) => ("bonus.glide_fall", format!("{v:+}")),
        Bonus::HammerStunMs(v) => ("bonus.hammer_stun_ms", v.to_string()),
        Bonus::HammerDamage(v) => ("bonus.hammer_damage", format!("{v:+}")),
        Bonus::HammerReachPct(v) => ("bonus.hammer_reach_pct", format!("{v:+}")),
        Bonus::HammerShockwave(_) => ("bonus.hammer_shockwave", String::new()),
        Bonus::ExplosionPct(v) => ("bonus.explosion_pct", format!("{v:+}")),
        Bonus::GrenadeShards(v) => ("bonus.grenade_shards", v.to_string()),
        Bonus::LaserBounces(v) => ("bonus.laser_bounces", format!("{v:+}")),
        Bonus::LaserPierce(v) => ("bonus.laser_pierce", v.to_string()),
        Bonus::LaserDelayPct(v) => ("bonus.laser_delay_pct", format!("{v:+}")),
    };
    lang.f(key, &[("n", &n)])
}

/// Symbol eines Gegenstands nach Art (Entwurf `abenteuer-ui.png`): Trank, Hut, Umhang,
/// Stiefel, Anhänger, Material-Brocken; Glanztropfen und Sammelstücke aus den Grafiken.
pub fn item_icon(
    batch: &mut elora_render::ShapeBatch,
    d: &MenuData<'_>,
    id: &str,
    c: Vec2,
    scale: f32,
) {
    let k = scale;
    let line = ui::OUTLINE;
    let kind = d.content.item(id).map(|i| &i.kind);
    let material_color = |id: &str| match id {
        "bernstein" => Color::hex(0xe0b85a),
        "harz" => Color::hex(0xc9955c),
        "glutstein" => Color::hex(0xe8685a),
        "eiskristall" => Color::hex(0xbfe6f5),
        "sternsplitter" => Color::hex(0xa77be0),
        _ => Color::hex(0x9aa4ae),
    };
    match kind {
        Some(ItemKind::Consumable { effect }) => {
            let color = match effect {
                elora_adventure::data::Effect::Heal(_) => Color::hex(0xe05a7a),
                elora_adventure::data::Effect::Tau(_) => Color::hex(0x5aaee8),
            };
            batch.fill_rect(
                c + Vec2::new(-3.5, -13.5) * k,
                c + Vec2::new(3.5, -6.0) * k,
                line,
            );
            batch.fill_rect(
                c + Vec2::new(-2.2, -12.2) * k,
                c + Vec2::new(2.2, -6.5) * k,
                Color::hex(0xa8744a),
            );
            batch.fill_circle(c + Vec2::new(0.0, 2.0) * k, 10.0 * k, line);
            batch.fill_circle(c + Vec2::new(0.0, 2.0) * k, 8.5 * k, color);
            batch.fill_circle(
                c + Vec2::new(-3.5, -1.5) * k,
                2.2 * k,
                Color::rgba(1.0, 1.0, 1.0, 0.7),
            );
        }
        Some(ItemKind::Equipment { slot, .. }) => match slot {
            Slot::Hat => {
                batch.fill_rounded_rect(
                    c + Vec2::new(-15.0, 3.0) * k,
                    c + Vec2::new(15.0, 9.0) * k,
                    3.0 * k,
                    line,
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-14.0, 4.0) * k,
                    c + Vec2::new(14.0, 8.0) * k,
                    2.0 * k,
                    Color::hex(0xe0c89a),
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-9.0, -9.0) * k,
                    c + Vec2::new(9.0, 5.0) * k,
                    6.0 * k,
                    line,
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-8.0, -8.0) * k,
                    c + Vec2::new(8.0, 4.0) * k,
                    5.0 * k,
                    Color::hex(0xe0c89a),
                );
                batch.fill_rect(
                    c + Vec2::new(-8.0, -1.0) * k,
                    c + Vec2::new(8.0, 2.0) * k,
                    Color::hex(0x6cbf4a),
                );
            }
            Slot::Cape => {
                batch.fill_rounded_rect(
                    c + Vec2::new(-11.0, -11.0) * k,
                    c + Vec2::new(11.0, 12.0) * k,
                    5.0 * k,
                    line,
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-9.5, -9.5) * k,
                    c + Vec2::new(9.5, 10.5) * k,
                    4.0 * k,
                    Color::hex(0x5aaee8),
                );
                batch.fill_rect(
                    c + Vec2::new(-9.5, -9.5) * k,
                    c + Vec2::new(9.5, -6.0) * k,
                    Color::hex(0x3d86c0),
                );
            }
            Slot::Boots => {
                batch.fill_rounded_rect(
                    c + Vec2::new(-9.0, -11.0) * k,
                    c + Vec2::new(1.0, 8.0) * k,
                    2.0 * k,
                    line,
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-9.0, 2.0) * k,
                    c + Vec2::new(12.0, 9.0) * k,
                    3.0 * k,
                    line,
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-7.5, -9.5) * k,
                    c + Vec2::new(-0.5, 7.5) * k,
                    1.5 * k,
                    Color::hex(0xe0c89a),
                );
                batch.fill_rounded_rect(
                    c + Vec2::new(-7.5, 3.5) * k,
                    c + Vec2::new(10.5, 7.5) * k,
                    2.0 * k,
                    Color::hex(0xe0c89a),
                );
            }
            Slot::Pendant => {
                batch.stroke_line(
                    c + Vec2::new(-8.0, -10.0) * k,
                    c + Vec2::new(0.0, -2.0) * k,
                    1.6 * k,
                    line,
                );
                batch.stroke_line(
                    c + Vec2::new(8.0, -10.0) * k,
                    c + Vec2::new(0.0, -2.0) * k,
                    1.6 * k,
                    line,
                );
                batch.fill_circle(c + Vec2::new(0.0, 3.0) * k, 7.0 * k, line);
                batch.fill_circle(c + Vec2::new(0.0, 3.0) * k, 5.5 * k, Color::hex(0xef7fb0));
            }
        },
        Some(ItemKind::Material) => {
            let col = material_color(id);
            batch.fill_circle(c, 10.5 * k, line);
            batch.fill_circle(c, 9.0 * k, col);
            batch.fill_circle(
                c + Vec2::new(-3.0, -3.0) * k,
                2.5 * k,
                Color::rgba(1.0, 1.0, 1.0, 0.55),
            );
        }
        _ => d.art.draw_loot_icon(batch, id, c, 1.1 * k),
    }
}

fn weapon_key(w: Weapon) -> &'static str {
    match w {
        Weapon::Hammer => "adventure.weapon_hammer",
        Weapon::Grenade => "adventure.weapon_grenade",
        Weapon::Laser => "adventure.weapon_laser",
    }
}

fn slot_key(s: Slot) -> &'static str {
    match s {
        Slot::Hat => "adventure.slot_hat",
        Slot::Cape => "adventure.slot_cape",
        Slot::Boots => "adventure.slot_boots",
        Slot::Pendant => "adventure.slot_pendant",
    }
}

/// Glanztropfen und Stufe oben rechts im Menü.
fn purse(ui: &mut Ui<'_>, d: &MenuData<'_>, card: Rect) {
    let s = ui.s;
    let p = Vec2::new(card.max.x - 160.0 * s, card.min.y + 30.0 * s);
    d.art.draw_loot_icon(ui.batch, GLANZTROPFEN, p, 1.0 * s);
    ui.label(
        &d.save.glanztropfen.to_string(),
        p + Vec2::new(14.0 * s, 0.0),
        13.0,
        ui::TEXT,
        Align::Left,
    );
    let c = Vec2::new(card.max.x - 80.0 * s, p.y);
    ui.batch.fill_circle(c, 13.0 * s, ui::OUTLINE);
    ui.batch.fill_circle(c, 11.5 * s, Color::hex(0xf2c14e));
    ui.label(&d.save.level.to_string(), c, 12.0, ui::TEXT, Align::Center);
}

/// Menü zeichnen; liefert einen Befehl.
pub fn draw(ui: &mut Ui<'_>, d: &MenuData<'_>, st: &mut MenuState) -> Option<Command> {
    let s = ui.s;
    ui.batch
        .fill_rect(Vec2::ZERO, d.screen, Color::rgba(0.118, 0.165, 0.212, 0.35));
    let w = (d.screen.x - 60.0 * s).min(980.0 * s);
    let h = (d.screen.y - 60.0 * s).min(560.0 * s);
    let card = Rect::new((d.screen.x - w) / 2.0, (d.screen.y - h) / 2.0, w, h);
    ui.card(card);
    let lang = d.lang;
    let tabs = [
        lang.t("adventure.menu_inventory"),
        lang.t("adventure.menu_skills"),
        lang.t("adventure.menu_quests"),
        lang.t("adventure.menu_map"),
    ];
    let mut cmd = None;
    match &st.panel {
        Panel::Shop(_) | Panel::Forge => {}
        p => {
            let selected = match p {
                Panel::Skills => 1,
                Panel::Quests => 2,
                Panel::Map => 3,
                _ => 0,
            };
            let origin = card.min + Vec2::new(18.0 * s, 16.0 * s);
            if let Some(i) = ui.tabs(
                "advmenu",
                origin,
                28.0 * s,
                &tabs,
                selected,
                &[TAB_COLORS[selected]],
            ) {
                st.panel = [Panel::Inventory, Panel::Skills, Panel::Quests, Panel::Map][i].clone();
                st.refusal = None;
            }
        }
    }
    purse(ui, d, card);
    let close = Rect::new(
        card.max.x - 50.0 * s,
        card.min.y + 16.0 * s,
        30.0 * s,
        28.0 * s,
    );
    if ui.button("advclose", close, "×", ui::GRAY) {
        cmd = Some(Command::Close);
    }
    let body = Rect::new(
        card.min.x + 18.0 * s,
        card.min.y + 60.0 * s,
        w - 36.0 * s,
        h - 90.0 * s,
    );
    let panel = st.panel.clone();
    let c = match &panel {
        Panel::Inventory => inventory(ui, d, st, body),
        Panel::Skills => skills(ui, d, st, body),
        Panel::Quests => quests(ui, d, st, body),
        Panel::Map => {
            world_map(ui, d, body);
            None
        }
        Panel::Shop(id) => shop(ui, d, st, body, id),
        Panel::Forge => forge(ui, d, body),
    };
    cmd = cmd.or(c);
    let foot = Vec2::new(card.max.x - 18.0 * s, card.max.y - 16.0 * s);
    ui.label(
        lang.t("adventure.menu_close"),
        foot,
        10.0,
        ui::TEXT_DIM,
        Align::Right,
    );
    if let Some(r) = st.refusal {
        let text = lang.f("adventure.refused", &[("why", &lang.t(refusal_key(r)))]);
        ui.label(
            &text,
            Vec2::new(card.min.x + 18.0 * s, foot.y),
            11.0,
            PRICE_BAD,
            Align::Left,
        );
    }
    cmd
}

/// Gegenstände im Rucksack (ohne Währung), sortiert nach Art und Name.
fn backpack<'a>(d: &MenuData<'a>) -> Vec<(&'a str, u32)> {
    let mut v: Vec<(&str, u32)> = d
        .save
        .inventory
        .iter()
        .filter(|(_, n)| **n > 0)
        .filter_map(|(id, n)| Some((d.content.items.get_key_value(id)?.0.as_str(), *n)))
        .collect();
    v.sort_by_key(|(id, _)| {
        let rank = match d.content.item(id).map(|i| &i.kind) {
            Some(ItemKind::Equipment { .. }) => 0,
            Some(ItemKind::Consumable { .. }) => 1,
            Some(ItemKind::Material) => 2,
            Some(ItemKind::Collectible) => 3,
            _ => 4,
        };
        (rank, *id)
    });
    v
}

fn slot_box(ui: &mut Ui<'_>, r: Rect, selected: bool) {
    let s = ui.s;
    let edge = if selected { ui::TEXT } else { ui::CARD_EDGE };
    let width = if selected { 2.5 } else { 1.5 } * s;
    ui.batch.fill_rounded_rect(
        r.min - Vec2::new(width, width),
        r.max + Vec2::new(width, width),
        10.0 * s + width,
        edge,
    );
    ui.batch
        .fill_rounded_rect(r.min, r.max, 10.0 * s, ui::FIELD);
}

fn inventory(ui: &mut Ui<'_>, d: &MenuData<'_>, st: &mut MenuState, area: Rect) -> Option<Command> {
    let s = ui.s;
    let lang = d.lang;
    let mut cmd = None;
    // Elora mit Ausrüstung
    let center = Vec2::new(area.min.x + 120.0 * s, area.min.y + 170.0 * s);
    ui.batch
        .fill_circle(center, 90.0 * s, Color::rgba(0.95, 0.76, 0.31, 0.15));
    d.figure.draw_pose(
        ui.batch,
        center + Vec2::new(0.0, 70.0 * s),
        140.0 * s,
        1.0,
        d.tint,
    );
    let places = [
        (Slot::Hat, Vec2::new(-110.0, -110.0)),
        (Slot::Cape, Vec2::new(60.0, -110.0)),
        (Slot::Boots, Vec2::new(-110.0, 70.0)),
        (Slot::Pendant, Vec2::new(60.0, 70.0)),
    ];
    for (slot, off) in places {
        let r = Rect::new(
            center.x + off.x * s,
            center.y + off.y * s,
            50.0 * s,
            50.0 * s,
        );
        let worn = d.save.equipped.get(&slot);
        slot_box(ui, r, worn.is_some_and(|w| st.item.as_ref() == Some(w)));
        if let Some(id) = worn {
            item_icon(ui.batch, d, id, r.center(), 1.3 * s);
            if ui.click(&format!("slot{slot:?}"), r) {
                st.item = Some(id.clone());
            }
        }
        ui.label(
            lang.t(slot_key(slot)),
            Vec2::new(r.center().x, r.max.y + 10.0 * s),
            9.0,
            ui::TEXT_DIM,
            Align::Center,
        );
    }

    // Rucksack
    let x0 = area.min.x + 290.0 * s;
    ui.label(
        lang.t("adventure.backpack"),
        Vec2::new(x0, area.min.y + 8.0 * s),
        13.0,
        ui::TEXT,
        Align::Left,
    );
    let items = backpack(d);
    let cols = 8;
    for (i, (id, n)) in items.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x0 + (i % cols) as f32 * 54.0 * s,
            area.min.y + 24.0 * s + (i / cols) as f32 * 54.0 * s,
            46.0 * s,
            46.0 * s,
        );
        slot_box(ui, r, st.item.as_deref() == Some(*id));
        item_icon(ui.batch, d, id, r.center(), 1.2 * s);
        if *n > 1 {
            ui.label(
                &n.to_string(),
                r.max - Vec2::new(6.0 * s, 8.0 * s),
                10.0,
                ui::TEXT,
                Align::Right,
            );
        }
        if ui.click(&format!("inv{i}"), r) {
            st.item = Some((*id).to_owned());
            st.refusal = None;
        }
    }

    // Beschreibung
    let Some(id) = st.item.clone() else {
        return cmd;
    };
    let Some(def) = d.content.item(&id) else {
        return cmd;
    };
    let desc = Rect::new(x0, area.max.y - 150.0 * s, area.max.x - x0, 150.0 * s);
    slot_box(ui, desc, false);
    let x = desc.min.x + 16.0 * s;
    ui.label(
        def.name.get(d.code),
        Vec2::new(x, desc.min.y + 20.0 * s),
        14.0,
        ui::TEXT,
        Align::Left,
    );
    let mut y = desc.min.y + 40.0 * s;
    if let ItemKind::Equipment {
        rarity, bonuses, ..
    } = &def.kind
    {
        let key = match rarity {
            Rarity::Common => "adventure.rarity_common",
            Rarity::Rare => "adventure.rarity_rare",
            Rarity::Guardian => "adventure.rarity_guardian",
        };
        ui.label(lang.t(key), Vec2::new(x, y), 10.0, ui::BLUE, Align::Left);
        y += 20.0 * s;
        for b in bonuses {
            ui.label(
                &bonus_text(lang, *b),
                Vec2::new(x, y),
                11.0,
                ui::TEXT,
                Align::Left,
            );
            y += 16.0 * s;
        }
    }
    if !def.desc.get(d.code).is_empty() {
        ui.label(
            &format!("„{}“", def.desc.get(d.code)),
            Vec2::new(x, y + 4.0 * s),
            10.0,
            ui::TEXT_DIM,
            Align::Left,
        );
    }
    let button = Rect::new(
        desc.max.x - 130.0 * s,
        desc.max.y - 40.0 * s,
        114.0 * s,
        28.0 * s,
    );
    match &def.kind {
        ItemKind::Equipment { slot, .. } => {
            if d.save.equipped.get(slot) == Some(&id) {
                if ui.button("unequip", button, lang.t("adventure.unequip"), ui::GRAY) {
                    cmd = Some(Command::Unequip(*slot));
                }
            } else if ui.button("equip", button, lang.t("adventure.equip"), ui::GREEN) {
                cmd = Some(Command::Equip(id.clone()));
            }
        }
        ItemKind::Consumable { .. }
            if ui.button("use", button, lang.t("adventure.use"), ui::GREEN) =>
        {
            cmd = Some(Command::Use(id.clone()));
        }
        _ => {}
    }
    cmd
}

fn skills(ui: &mut Ui<'_>, d: &MenuData<'_>, st: &mut MenuState, area: Rect) -> Option<Command> {
    let s = ui.s;
    let lang = d.lang;
    let mut cmd = None;
    ui.label(
        &lang.f("adventure.points", &[("n", &d.save.free_points())]),
        Vec2::new(area.min.x, area.min.y + 8.0 * s),
        13.0,
        ui::TEXT,
        Align::Left,
    );
    ui.label(
        lang.t("adventure.skills_hint"),
        Vec2::new(area.max.x, area.min.y + 8.0 * s),
        10.0,
        ui::TEXT_DIM,
        Align::Right,
    );
    let branches = [
        (Branch::Movement, "adventure.branch_movement", ui::BLUE),
        (
            Branch::Combat,
            "adventure.branch_combat",
            Color::hex(0xe8685a),
        ),
        (
            Branch::Spring,
            "adventure.branch_spring",
            Color::hex(0x3fc1b0),
        ),
    ];
    let col_w = area.w() / 3.0;
    let mut hover_desc = None;
    for (b, (branch, key, color)) in branches.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let cx = area.min.x + col_w * b as f32 + 40.0 * s;
        let head = Rect::new(cx - 20.0 * s, area.min.y + 28.0 * s, 130.0 * s, 24.0 * s);
        ui.batch
            .fill_rounded_rect(head.min, head.max, 12.0 * s, *color);
        ui.label(
            lang.t(key),
            head.center(),
            11.0,
            Color::rgb(1.0, 1.0, 1.0),
            Align::Center,
        );
        let nodes: Vec<_> = SaveGame::branch(d.content, *branch).collect();
        for (i, n) in nodes.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let y = area.min.y + (82.0 + i as f32 * 54.0) * s;
            let c = Vec2::new(cx, y);
            let rank = d.save.skills.get(&n.id).copied().unwrap_or(0);
            let can = d.save.can_learn(d.content, &n.id);
            if i + 1 < nodes.len() {
                let line = if rank > 0 { *color } else { LOCKED };
                ui.batch.stroke_line(
                    c + Vec2::new(0.0, 16.0 * s),
                    c + Vec2::new(0.0, 38.0 * s),
                    4.0 * s,
                    line,
                );
            }
            let fill = if rank > 0 { *color } else { ui::FIELD };
            let edge = if rank > 0 || can.is_ok() {
                ui::OUTLINE
            } else {
                LOCKED
            };
            ui.batch.fill_circle(c, 16.5 * s, edge);
            ui.batch.fill_circle(c, 14.5 * s, fill);
            if can.is_ok() {
                ui.batch
                    .stroke_circle(c, 20.0 * s, 3.0 * s, Color::hex(0xf2c14e));
                ui.label("+", c, 16.0, ui::TEXT, Align::Center);
            } else if rank > 0 {
                ui.label(
                    &rank.to_string(),
                    c,
                    12.0,
                    Color::rgb(1.0, 1.0, 1.0),
                    Align::Center,
                );
            }
            let name_color = if rank > 0 || can.is_ok() {
                ui::TEXT
            } else {
                ui::TEXT_DIM
            };
            ui.label(
                n.name.get(d.code),
                c + Vec2::new(26.0 * s, -6.0 * s),
                11.0,
                name_color,
                Align::Left,
            );
            ui.label(
                &lang.f("adventure.rank", &[("r", &rank), ("max", &n.ranks)]),
                c + Vec2::new(26.0 * s, 8.0 * s),
                9.0,
                ui::TEXT_DIM,
                Align::Left,
            );
            let hit = Rect::new(c.x - 18.0 * s, c.y - 18.0 * s, col_w - 30.0 * s, 36.0 * s);
            if ui.hovered(hit) {
                hover_desc = Some((*n, can));
            }
            if ui.click(&format!("skill{}", n.id), hit) {
                match can {
                    Ok(()) => cmd = Some(Command::Learn(n.id.clone())),
                    Err(r) => st.refusal = Some(r),
                }
            }
        }
    }
    if let Some((n, _)) = hover_desc {
        let mut text = n.desc.get(d.code).to_owned();
        let effects: Vec<String> = n.per_rank.iter().map(|b| bonus_text(lang, *b)).collect();
        if !effects.is_empty() {
            text = format!("{text}  ·  {}", effects.join(", "));
        }
        ui.label(
            &text,
            Vec2::new(area.min.x, area.max.y - 6.0 * s),
            11.0,
            ui::TEXT,
            Align::Left,
        );
    }
    cmd
}

fn quests(ui: &mut Ui<'_>, d: &MenuData<'_>, st: &mut MenuState, area: Rect) -> Option<Command> {
    let s = ui.s;
    let lang = d.lang;
    let mut list: Vec<_> = d
        .content
        .quests
        .iter()
        .filter_map(|q| Some((q, d.save.quest(&q.id)?)))
        .collect();
    list.sort_by_key(|(q, st)| (st.status != QuestStatus::Active, q.kind != QuestKind::Main));
    if list.is_empty() {
        ui.label(
            lang.t("adventure.quests_none"),
            area.min + Vec2::new(0.0, 12.0 * s),
            13.0,
            ui::TEXT_DIM,
            Align::Left,
        );
        return None;
    }
    if st
        .quest
        .as_ref()
        .is_none_or(|id| !list.iter().any(|(q, _)| &q.id == id))
    {
        st.quest = Some(list[0].0.id.clone());
    }
    for (i, (q, qs)) in list.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            area.min.x,
            area.min.y + i as f32 * 50.0 * s,
            260.0 * s,
            44.0 * s,
        );
        if st.quest.as_ref() == Some(&q.id) {
            ui.batch
                .fill_rounded_rect(r.min, r.max, 12.0 * s, Color::rgba(0.95, 0.76, 0.31, 0.35));
        }
        if ui.click(&format!("quest{i}"), r) {
            st.quest = Some(q.id.clone());
        }
        let dot = match qs.status {
            QuestStatus::Done => ui::GREEN,
            QuestStatus::Failed => ui::GRAY,
            QuestStatus::Active if q.kind == QuestKind::Main => Color::hex(0xf2c14e),
            QuestStatus::Active => ui::ORANGE,
        };
        ui.batch
            .fill_circle(r.min + Vec2::new(18.0 * s, 22.0 * s), 7.0 * s, ui::OUTLINE);
        ui.batch
            .fill_circle(r.min + Vec2::new(18.0 * s, 22.0 * s), 5.5 * s, dot);
        let source = match (qs.status, q.kind) {
            (QuestStatus::Done, _) => lang.t("adventure.quest_done_tag"),
            (QuestStatus::Failed, _) => lang.t("adventure.quest_failed_tag"),
            (_, QuestKind::Main) => lang.t("adventure.quest_main"),
            (_, QuestKind::Side) => lang.t("adventure.quest_side"),
        };
        ui.label(
            source,
            r.min + Vec2::new(34.0 * s, 15.0 * s),
            9.0,
            ui::TEXT_DIM,
            Align::Left,
        );
        ui.label(
            q.name.get(d.code),
            r.min + Vec2::new(34.0 * s, 31.0 * s),
            12.0,
            ui::TEXT,
            Align::Left,
        );
    }
    let (q, qs) = list
        .iter()
        .find(|(q, _)| Some(&q.id) == st.quest.as_ref())?;
    let box_ = Rect::new(
        area.min.x + 280.0 * s,
        area.min.y,
        area.w() - 280.0 * s,
        area.h(),
    );
    slot_box(ui, box_, false);
    let x = box_.min.x + 16.0 * s;
    ui.label(
        q.name.get(d.code),
        Vec2::new(x, box_.min.y + 24.0 * s),
        15.0,
        ui::TEXT,
        Align::Left,
    );
    let mut y = box_.min.y + 52.0 * s;
    for line in crate::adventure_hud::wrap(ui, q.desc.get(d.code), 11.0, box_.w() - 32.0 * s) {
        ui.label(&line, Vec2::new(x, y), 11.0, ui::TEXT, Align::Left);
        y += 16.0 * s;
    }
    y += 12.0 * s;
    // erledigte Schritte und der aktuelle, weitere als „?“ (E-251)
    for (i, step) in q.step.iter().enumerate() {
        let done = i < qs.step;
        if i > qs.step || (qs.status != QuestStatus::Active && !done) {
            ui.label("?   …", Vec2::new(x, y), 11.0, ui::TEXT_DIM, Align::Left);
            break;
        }
        let b = Rect::new(x, y - 6.0 * s, 12.0 * s, 12.0 * s);
        ui.batch
            .fill_rounded_rect(b.min, b.max, 3.0 * s, ui::OUTLINE);
        let inner = b.shrink(1.5 * s);
        ui.batch.fill_rounded_rect(
            inner.min,
            inner.max,
            2.0 * s,
            if done { ui::GREEN } else { ui::FIELD },
        );
        let mut text = step.text.get(d.code).to_owned();
        if !done && let elora_adventure::quest::Goal::Defeat { count, .. } = step.goal {
            text = format!("{text} ({}/{count})", qs.progress);
        }
        ui.label(
            &text,
            Vec2::new(x + 20.0 * s, y),
            11.0,
            if done { ui::TEXT_DIM } else { ui::TEXT },
            Align::Left,
        );
        y += 24.0 * s;
    }
    let r = &q.reward;
    let mut parts = Vec::new();
    if r.xp > 0 {
        parts.push(lang.f("adventure.xp", &[("n", &r.xp)]));
    }
    if r.glanztropfen > 0 {
        parts.push(lang.f("adventure.glanz", &[("n", &r.glanztropfen)]));
    }
    for c in &r.items {
        let name = d
            .content
            .item(&c.item)
            .map_or(c.item.as_str(), |i| i.name.get(d.code));
        parts.push(format!("{} × {name}", c.count));
    }
    if !parts.is_empty() {
        ui.label(
            &lang.f("adventure.reward", &[("r", &parts.join(" · "))]),
            Vec2::new(x, box_.max.y - 16.0 * s),
            10.0,
            ui::TEXT_DIM,
            Align::Left,
        );
    }
    None
}

fn hex(c: &str) -> Color {
    u32::from_str_radix(c.trim_start_matches('#'), 16).map_or(ui::GRAY, Color::hex)
}

/// Weltkarte des Taulands (E-264).
fn world_map(ui: &mut Ui<'_>, d: &MenuData<'_>, area: Rect) {
    let s = ui.s;
    let lang = d.lang;
    ui.batch
        .fill_rounded_rect(area.min, area.max, 16.0 * s, Color::hex(0xcfe6f2));
    let at = |p: [f32; 2]| Vec2::new(area.min.x + area.w() * p[0], area.min.y + area.h() * p[1]);
    let visited = |prefix: &str| {
        d.save
            .flags
            .keys()
            .filter_map(|k| k.strip_prefix("besucht:"))
            .filter(|m| m.starts_with(prefix))
            .count()
    };
    let here = d
        .content
        .area_of(&d.save.location.map)
        .map(|a| a.id.clone());
    let hub = d.content.areas.first().map(|a| at(a.pos));
    // Wege von Tauwinkel zu den Gebieten
    if let Some(hub) = hub {
        for a in d.content.areas.iter().skip(1) {
            let to = at(a.pos);
            let n = 14;
            for k in (0..n).step_by(2) {
                #[allow(clippy::cast_precision_loss)]
                let (t0, t1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
                ui.batch.stroke_line(
                    hub.lerp(to, t0),
                    hub.lerp(to, t1),
                    3.0 * s,
                    Color::hex(0xa8946e),
                );
            }
        }
    }
    for a in &d.content.areas {
        let p = at(a.pos);
        let known = visited(&a.maps) > 0;
        let color = if known {
            hex(&a.color)
        } else {
            Color::hex(0xc8c8c0)
        };
        ui.batch.fill_circle(p, 46.0 * s, ui::OUTLINE);
        ui.batch.fill_circle(p, 44.0 * s, color);
        ui.batch.fill_circle(
            p + Vec2::new(-12.0 * s, -14.0 * s),
            12.0 * s,
            Color::rgba(1.0, 1.0, 1.0, 0.25),
        );
        ui.label(
            a.name.get(d.code),
            p + Vec2::new(0.0, 60.0 * s),
            12.0,
            ui::TEXT,
            Align::Center,
        );
        // befreite Quelle: Lichtkranz und Hinweis
        let freed = a.freed(d.save);
        if freed {
            for k in 0..8 {
                #[allow(clippy::cast_precision_loss)]
                let ang = k as f32 * std::f32::consts::TAU / 8.0;
                let q = p + Vec2::new(ang.cos(), ang.sin()) * 54.0 * s;
                ui.batch.fill_circle(q, 5.0 * s, Color::hex(0xfff2b0));
            }
        }
        let sub = if !known {
            lang.t("adventure.map_unknown").to_owned()
        } else if freed {
            lang.t("adventure.map_freed").to_owned()
        } else if a.id == d.content.areas[0].id {
            String::new()
        } else {
            lang.f("adventure.map_sections", &[("n", &visited(&a.maps))])
        };
        ui.label(
            &sub,
            p + Vec2::new(0.0, 76.0 * s),
            9.0,
            ui::TEXT_DIM,
            Align::Center,
        );
        if here.as_deref() == Some(a.id.as_str()) {
            let tint = d.tint;
            d.figure
                .draw_pose(ui.batch, p + Vec2::new(0.0, 18.0 * s), 44.0 * s, 1.0, tint);
            ui.label(
                lang.t("adventure.map_here"),
                p - Vec2::new(0.0, 56.0 * s),
                10.0,
                ui::TEXT,
                Align::Center,
            );
        }
    }
}

fn shop(
    ui: &mut Ui<'_>,
    d: &MenuData<'_>,
    st: &mut MenuState,
    area: Rect,
    id: &str,
) -> Option<Command> {
    let s = ui.s;
    let lang = d.lang;
    let mut cmd = None;
    let sh = d.content.shops.get(id)?;
    // Ladenbesitzerin links
    let owner = sh.owner.as_deref().unwrap_or("lotte");
    let c = Vec2::new(area.min.x + 90.0 * s, area.min.y + 140.0 * s);
    ui.batch
        .fill_circle(c, 70.0 * s, Color::rgba(0.56, 0.82, 0.94, 0.25));
    d.art
        .draw_character(ui.batch, owner, c + Vec2::new(0.0, 60.0 * s), 1, 2.6 * s);
    if let Some(ch) = d.content.characters.get(owner) {
        ui.label(
            ch.name.get(d.code),
            c + Vec2::new(0.0, 96.0 * s),
            13.0,
            ui::TEXT,
            Align::Center,
        );
    }
    let x0 = area.min.x + 200.0 * s;
    for (i, (key, selling)) in [("adventure.shop_buy", false), ("adventure.shop_sell", true)]
        .into_iter()
        .enumerate()
    {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x0 + i as f32 * 110.0 * s,
            area.min.y - 44.0 * s,
            100.0 * s,
            26.0 * s,
        );
        let color = if st.selling == selling {
            ui::ORANGE
        } else {
            ui::SAND
        };
        if ui.button(&format!("shoptab{i}"), r, lang.t(key), color) {
            st.selling = selling;
            st.refusal = None;
        }
    }
    let rows: Vec<(String, u32)> = if st.selling {
        backpack(d)
            .into_iter()
            .filter_map(|(id, _)| Some((id.to_owned(), SaveGame::sell_price(d.content, id)?)))
            .collect()
    } else {
        sh.stock
            .iter()
            .filter_map(|it| Some((it.clone(), d.save.price(d.content, id, it)?)))
            .collect()
    };
    if rows.is_empty() {
        ui.label(
            lang.t("adventure.shop_nothing"),
            Vec2::new(x0, area.min.y + 16.0 * s),
            12.0,
            ui::TEXT_DIM,
            Align::Left,
        );
    }
    for (i, (it, price)) in rows.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x0,
            area.min.y + i as f32 * 48.0 * s,
            area.max.x - x0,
            42.0 * s,
        );
        slot_box(ui, r, false);
        item_icon(
            ui.batch,
            d,
            it,
            r.min + Vec2::new(24.0 * s, 21.0 * s),
            1.0 * s,
        );
        let def = d.content.item(it);
        ui.label(
            def.map_or(it.as_str(), |x| x.name.get(d.code)),
            r.min + Vec2::new(48.0 * s, 15.0 * s),
            12.0,
            ui::TEXT,
            Align::Left,
        );
        let note = def.map_or(String::new(), |x| match &x.kind {
            ItemKind::Equipment { bonuses, .. } => bonuses
                .iter()
                .map(|b| bonus_text(lang, *b))
                .collect::<Vec<_>>()
                .join(", "),
            _ => x.desc.get(d.code).to_owned(),
        });
        ui.label(
            &note,
            r.min + Vec2::new(48.0 * s, 30.0 * s),
            9.0,
            ui::TEXT_DIM,
            Align::Left,
        );
        let affordable = st.selling || d.save.glanztropfen >= *price;
        let pc = Vec2::new(r.max.x - 150.0 * s, r.center().y);
        ui.label(
            &price.to_string(),
            pc,
            12.0,
            if affordable { ui::TEXT } else { PRICE_BAD },
            Align::Right,
        );
        d.art.draw_loot_icon(
            ui.batch,
            GLANZTROPFEN,
            pc + Vec2::new(12.0 * s, 0.0),
            0.8 * s,
        );
        let b = Rect::new(r.max.x - 110.0 * s, r.min.y + 8.0 * s, 96.0 * s, 26.0 * s);
        let (label, color) = if st.selling {
            (lang.t("adventure.shop_sell"), ui::SAND)
        } else {
            (
                lang.t("adventure.shop_buy"),
                if affordable { ui::GREEN } else { ui::GRAY },
            )
        };
        if ui.button(&format!("shop{i}"), b, label, color) {
            cmd = Some(if st.selling {
                Command::Sell(it.clone())
            } else {
                Command::Buy(id.to_owned(), it.clone())
            });
        }
    }
    cmd
}

fn forge(ui: &mut Ui<'_>, d: &MenuData<'_>, area: Rect) -> Option<Command> {
    let s = ui.s;
    let lang = d.lang;
    let mut cmd = None;
    let c = Vec2::new(area.min.x + 90.0 * s, area.min.y + 140.0 * s);
    ui.batch
        .fill_circle(c, 70.0 * s, Color::rgba(0.66, 0.45, 0.29, 0.2));
    d.art
        .draw_character(ui.batch, "klonk", c + Vec2::new(0.0, 60.0 * s), 1, 2.3 * s);
    ui.label(
        lang.t("adventure.forge"),
        c + Vec2::new(0.0, 96.0 * s),
        13.0,
        ui::TEXT,
        Align::Center,
    );
    let x0 = area.min.x + 200.0 * s;
    for (i, w) in Weapon::ALL.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x0,
            area.min.y + i as f32 * 110.0 * s,
            area.max.x - x0,
            100.0 * s,
        );
        slot_box(ui, r, false);
        let x = r.min.x + 16.0 * s;
        ui.label(
            lang.t(weapon_key(*w)),
            Vec2::new(x, r.min.y + 20.0 * s),
            14.0,
            ui::TEXT,
            Align::Left,
        );
        let Some(&level) = d.save.weapons.get(w) else {
            ui.label(
                lang.t("adventure.not_owned"),
                Vec2::new(x, r.min.y + 44.0 * s),
                11.0,
                ui::TEXT_DIM,
                Align::Left,
            );
            continue;
        };
        ui.label(
            &lang.f("adventure.forge_level", &[("n", &level)]),
            Vec2::new(r.max.x - 16.0 * s, r.min.y + 20.0 * s),
            11.0,
            ui::TEXT_DIM,
            Align::Right,
        );
        let Some(u) = d.content.upgrade(*w, level + 1) else {
            ui.label(
                lang.t("adventure.forge_max"),
                Vec2::new(x, r.min.y + 46.0 * s),
                11.0,
                ui::TEXT_DIM,
                Align::Left,
            );
            continue;
        };
        ui.label(
            &format!("{} – {}", u.name.get(d.code), u.desc.get(d.code)),
            Vec2::new(x, r.min.y + 44.0 * s),
            12.0,
            ui::TEXT,
            Align::Left,
        );
        // Kosten: Glanztropfen und Material mit Bestand
        let mut cx = x;
        let y = r.min.y + 72.0 * s;
        let enough = d.save.glanztropfen >= u.glanztropfen;
        d.art
            .draw_loot_icon(ui.batch, GLANZTROPFEN, Vec2::new(cx + 6.0 * s, y), 0.8 * s);
        ui.label(
            &u.glanztropfen.to_string(),
            Vec2::new(cx + 16.0 * s, y),
            11.0,
            if enough { ui::TEXT } else { PRICE_BAD },
            Align::Left,
        );
        cx += 70.0 * s;
        for m in &u.materials {
            let have = d.save.count(&m.item);
            let name = d
                .content
                .item(&m.item)
                .map_or(m.item.as_str(), |i| i.name.get(d.code));
            let text = format!("{name} {have}/{}", m.count);
            ui.label(
                &text,
                Vec2::new(cx, y),
                11.0,
                if have >= m.count { ui::TEXT } else { PRICE_BAD },
                Align::Left,
            );
            cx += ui.text_width(&text, 11.0) + 20.0 * s;
        }
        let b = Rect::new(r.max.x - 126.0 * s, r.max.y - 38.0 * s, 110.0 * s, 28.0 * s);
        if ui.button(
            &format!("forge{i}"),
            b,
            lang.t("adventure.upgrade"),
            ui::GREEN,
        ) {
            cmd = Some(Command::Upgrade(*w));
        }
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::Language;
    use elora_adventure::Location;
    use elora_render::{Font, ShapeBatch};

    /// Sichtprüfung: `cargo test -p elora-client --bin elora adventure_menu_sheet -- --ignored`,
    /// dann je Seite `cargo xtask svg-preview target/abenteuer-menu-<seite>.svg … 1280`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn adventure_menu_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let art = CreatureArt::load();
        let figure = FigureArt::load();
        let c = Content::builtin();
        let mut save = SaveGame::new(
            &c,
            Location {
                map: "wiese-1".into(),
                spawn: "west".into(),
            },
        );
        save.add_xp(&c, 400);
        save.grant_ability(elora_sim::Ability::HookRuck);
        save.learn(&c, "schneller_ruck").unwrap();
        save.learn(&c, "kraft").unwrap();
        save.learn(&c, "kraft").unwrap();
        for (id, n) in [
            ("glanztropfen", 140),
            ("heiltrank", 3),
            ("bernstein", 2),
            ("tauumhang", 1),
            ("glitzerstein", 1),
            ("strohhut", 1),
        ] {
            save.add_item(&c, id, n).unwrap();
        }
        save.equip(&c, "strohhut").unwrap();
        save.give_weapon(Weapon::Grenade);
        save.set_flag("besucht:tauwinkel", 1);
        save.set_flag("besucht:wiese-1", 1);
        save.run(
            &c,
            &[
                "quest brunnen start".into(),
                "quest brunnen weiter".into(),
                "quest pips_stein start".into(),
            ],
        );
        let tint = crate::skins::tint(
            elora_protocol::Skin::default(),
            elora_sim::Team::None,
            false,
            crate::draw::team_color,
        );
        let screen = Vec2::new(1280.0, 720.0);
        for (name, panel) in [
            ("inventar", Panel::Inventory),
            ("faehigkeiten", Panel::Skills),
            ("aufgaben", Panel::Quests),
            ("karte", Panel::Map),
            ("laden", Panel::Shop("lotte".into())),
            ("schmiede", Panel::Forge),
        ] {
            let mut batch = ShapeBatch::default();
            batch.fill_rect(Vec2::ZERO, screen, Color::hex(0x8fbf7a));
            let input = crate::ui::UiInput::default();
            let mut state = crate::ui::UiState::default();
            let mut ui = Ui {
                batch: &mut batch,
                font: &font,
                input: &input,
                state: &mut state,
                s: 1.0,
            };
            ui.begin(0.016);
            let data = MenuData {
                lang: &lang,
                code: "de",
                content: &c,
                save: &save,
                art: &art,
                figure: &figure,
                tint: &tint,
                screen,
            };
            let mut st = MenuState {
                panel,
                item: Some("tauumhang".into()),
                ..MenuState::default()
            };
            draw(&mut ui, &data, &mut st);
            ui.end();
            let svg = batch.debug_svg(Vec2::ZERO, screen, Color::hex(0x8fbf7a));
            std::fs::write(
                format!(
                    "{}/../../target/abenteuer-menu-{name}.svg",
                    env!("CARGO_MANIFEST_DIR")
                ),
                svg,
            )
            .unwrap();
        }
    }
}
