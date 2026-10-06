//! Seitenleiste und Kartenfläche des Werkzeugs „Abenteuer“ (A1.8, E-268 bis E-271):
//! Art wählen, Eigenschaften bearbeiten, Gespräche ansehen und testen, Fehler, Teststand.

use std::time::Instant;

use egui::Ui;
use elora_adventure::{Content, Conversation, Location, SaveGame};
use elora_map::adventure::{CameraMode, SwitchTrigger};
use elora_map::{Map, ObjectKind};
use elora_sim::{Ability, TILE_SIZE, Vec2, Weapon};

use super::Editor;
use super::adventure::Kind;
use super::panel::{AreaInfo, Preview};
use super::tools::Cells;
use crate::lang::Lang;

/// Teststand für F5 (E-269).
#[derive(Debug, Clone)]
pub struct TestSetup {
    pub level: u32,
    pub abilities: elora_sim::Abilities,
    pub grenade: bool,
    pub laser: bool,
    /// Merker wie `tor.dorf=1, oma.frech`.
    pub flags: String,
    /// Start an diesem Objekt; `None` = an der Maus.
    pub start: Option<String>,
}

impl Default for TestSetup {
    fn default() -> Self {
        Self {
            level: 1,
            abilities: elora_sim::Abilities::NONE,
            grenade: false,
            laser: false,
            flags: String::new(),
            start: None,
        }
    }
}

impl TestSetup {
    /// Spielstand für das Testspiel und den Gesprächstest.
    pub fn save(&self, c: &Content, map: &str, spawn: &str) -> SaveGame {
        let mut g = SaveGame::new(
            c,
            Location {
                map: map.to_owned(),
                spawn: spawn.to_owned(),
            },
        );
        while g.level < self.level.min(c.progression.max_level) {
            let need = c.xp_to_next(g.level);
            g.add_xp(c, need);
        }
        g.abilities = self.abilities.bits();
        if self.grenade {
            g.give_weapon(Weapon::Grenade);
        }
        if self.laser {
            g.give_weapon(Weapon::Laser);
        }
        for part in self
            .flags
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            let (k, v) = part.split_once('=').unwrap_or((part, "1"));
            g.set_flag(k.trim(), v.trim().parse().unwrap_or(1));
        }
        g.health = g.max_health(c);
        g
    }
}

/// Gespräch im Testfenster (E-270).
#[derive(Debug, Clone)]
pub struct DialogTest {
    pub dialog: String,
    pub conv: Option<Conversation>,
    pub save: SaveGame,
    pub before: SaveGame,
}

/// Inhalte des Abenteuers aus `assets/adventure` (neu ladbar, E-270).
pub fn load_content() -> Result<Content, String> {
    let dir = elora_server::paths::resolve(std::path::Path::new("assets/adventure"));
    Content::from_dir(&dir).map_err(|e| e.to_string())
}

impl Editor {
    /// Inhalte laden, falls noch nicht geschehen; setzt die Vorgaben für neue Objekte.
    pub fn ensure_content(&mut self) {
        if self.adventure_content.is_none() {
            self.reload_content();
        }
    }

    pub fn reload_content(&mut self) {
        let r = load_content();
        if let Ok(c) = &r {
            let d = &mut self.adventure.defaults;
            d.creature = c
                .creatures
                .first()
                .map(|k| k.name.clone())
                .unwrap_or_default();
            d.character = c.characters.keys().next().cloned().unwrap_or_default();
            d.item = c
                .items
                .iter()
                .find(|(_, i)| matches!(i.kind, elora_adventure::data::ItemKind::Collectible))
                .map(|(k, _)| k.clone())
                .unwrap_or_default();
        }
        self.adventure_content = Some(r);
    }

    pub fn content(&self) -> Option<&Content> {
        self.adventure_content.as_ref()?.as_ref().ok()
    }

    /// Höhe eines Gegners aus `creatures.toml`.
    fn creature_height(&self, name: &str) -> Option<f32> {
        self.content()?
            .creatures
            .iter()
            .find(|k| k.name == name)
            .map(|k| k.size[1])
    }

    /// Eingänge und Speicherpunkte (Start im Testspiel).
    pub fn entrances(&self) -> Vec<String> {
        self.map
            .adventure
            .objects
            .iter()
            .filter(|o| matches!(o.kind, ObjectKind::Spawn | ObjectKind::SavePoint))
            .map(|o| o.id.clone())
            .collect()
    }
}

/// Abenteuer-Karten (eigene vor mitgelieferten) mit ihren Eingängen – für Übergänge und die
/// Prüfung der Verbindungen.
fn adventure_maps(editor: &Editor) -> Vec<(String, Map)> {
    let mut out: Vec<(String, Map)> = Vec::new();
    let dirs = editor
        .user_dir
        .iter()
        .map(|d| d.join("abenteuer"))
        .chain(std::iter::once(editor.bundled_dir.join("abenteuer")));
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
        files.sort();
        for p in files {
            let Some(id) = p.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            if out.iter().any(|(n, _)| *n == id) {
                continue;
            }
            if let Ok(m) = std::fs::read(&p)
                .map_err(|e| e.to_string())
                .and_then(|d| elora_map::decode_draft(&d).map_err(|e| e.to_string()))
            {
                out.push((id, m));
            }
        }
    }
    out
}

/// Kartenfläche: setzen, wählen, ziehen, aufziehen, löschen (rechts).
pub fn interact(
    editor: &mut Editor,
    info: &mut AreaInfo,
    world: Vec2,
    input: (bool, bool, bool),
    now: Instant,
) {
    let (pressed, primary, down) = input;
    editor.ensure_content();
    let Some((x, y)) = info.hover else {
        if !down {
            editor.adventure.drag = None;
            editor.adventure.area_start = None;
        }
        return;
    };
    info.preview = Preview::Cells(Cells::span((x, y), (x, y)));
    if pressed && !primary {
        if let Some(id) = editor.object_at(world) {
            editor.remove_object(&id, now);
        }
        return;
    }
    if pressed {
        if let Some(id) = editor.object_at(world) {
            let pos = editor.map.adventure.object(&id).map_or(world, |o| o.pos);
            editor.adventure.selected = Some(id);
            editor.adventure.drag = Some((world, pos));
            editor.end_edit();
        } else if editor.adventure.kind.is_area() {
            editor.adventure.area_start = Some((x, y));
        } else {
            let heights: Vec<(String, f32)> = editor
                .content()
                .map(|c| {
                    c.creatures
                        .iter()
                        .map(|k| (k.name.clone(), k.size[1]))
                        .collect()
                })
                .unwrap_or_default();
            editor.place_object(
                x,
                y,
                |n| heights.iter().find(|(k, _)| k == n).map(|(_, h)| *h),
                now,
            );
        }
    }
    if let Some(start) = editor.adventure.area_start {
        info.preview = Preview::Cells(Cells::span(start, (x, y)));
        if !down {
            editor.place_area(start, (x, y), now);
            editor.adventure.area_start = None;
        }
    }
    if let Some((from, orig)) = editor.adventure.drag {
        if down {
            if world.distance(from) > 4.0 {
                editor.move_object(orig, world - from, now);
            }
        } else {
            editor.adventure.drag = None;
            editor.end_edit();
        }
    }
    let _ = editor.creature_height("");
}

fn combo(ui: &mut Ui, id: &str, label: &str, value: &mut String, options: &[String]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(value.as_str())
            .show_ui(ui, |ui| {
                for o in options {
                    if ui.selectable_label(value == o, o).clicked() && value != o {
                        value.clone_from(o);
                        changed = true;
                    }
                }
            });
    });
    changed
}

fn ability_name(a: Ability) -> &'static str {
    match a {
        Ability::HookRuck => "hook-ruck",
        Ability::Pull => "heranhooken",
        Ability::Stomp => "stampfen",
        Ability::Grip => "eisgriff",
        Ability::Glide => "gleiten",
    }
}

/// Seitenleiste des Werkzeugs.
pub fn tool(ui: &mut Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    editor.ensure_content();
    ui.horizontal_wrapped(|ui| {
        for k in Kind::ALL {
            if ui
                .selectable_label(editor.adventure.kind == k, lang.t(k.key()))
                .clicked()
            {
                editor.adventure.kind = k;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label(lang.t("editor.adv_map_id"));
        ui.text_edit_singleline(&mut editor.adventure_id);
    });
    match &editor.adventure_content {
        Some(Err(e)) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                lang.f("editor.adv_content_error", &[("arg", e)]),
            );
        }
        Some(Ok(c)) => {
            ui.small(lang.f(
                "editor.adv_content_ok",
                &[(
                    "arg",
                    &format!(
                        "{} / {} / {}",
                        c.creatures.len(),
                        c.characters.len(),
                        c.dialogs.len()
                    ),
                )],
            ));
        }
        None => {}
    }
    if ui.button(lang.t("editor.adv_reload")).clicked() {
        editor.reload_content();
    }
    ui.separator();
    properties(ui, editor, lang, now);
    ui.separator();
    checks(ui, editor, lang);
    ui.separator();
    test_setup(ui, editor, lang);
}

#[allow(clippy::too_many_lines)] // je Art ein kleines Formular
fn properties(ui: &mut Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    let Some(obj) = editor.selected_object().cloned() else {
        ui.small(lang.t("editor.adv_none_selected"));
        return;
    };
    let c = editor.content().cloned();
    let creatures: Vec<String> = c
        .iter()
        .flat_map(|c| c.creatures.iter().map(|k| k.name.clone()))
        .collect();
    let characters: Vec<String> = c
        .iter()
        .flat_map(|c| c.characters.keys().cloned())
        .collect();
    let dialogs: Vec<String> = c.iter().flat_map(|c| c.dialogs.keys().cloned()).collect();
    let items: Vec<String> = c.iter().flat_map(|c| c.items.keys().cloned()).collect();
    ui.strong(lang.t(Kind::of(&obj.kind).key()));
    let mut id = obj.id.clone();
    ui.horizontal(|ui| {
        ui.label("Id");
        let r = ui.text_edit_singleline(&mut id);
        if r.lost_focus() && id != obj.id && !editor.rename_object(&id, now) {
            editor.note("editor.adv_id_taken", id.clone());
        }
    });
    ui.small(format!("({:.0}, {:.0})", obj.pos.x, obj.pos.y));
    let mut kind = obj.kind.clone();
    let mut changed = false;
    match &mut kind {
        ObjectKind::Creature { kind, persistent } => {
            changed |= combo(
                ui,
                "adv_creature",
                lang.t("editor.adv_creature_kind"),
                kind,
                &creatures,
            );
            changed |= ui
                .checkbox(persistent, lang.t("editor.adv_persistent"))
                .changed();
        }
        ObjectKind::Npc {
            character,
            dialog,
            facing,
            walk,
        } => {
            changed |= combo(
                ui,
                "adv_char",
                lang.t("editor.adv_character"),
                character,
                &characters,
            );
            changed |= combo(
                ui,
                "adv_dialog",
                lang.t("editor.adv_dialog"),
                dialog,
                &dialogs,
            );
            ui.horizontal(|ui| {
                changed |= ui
                    .radio_value(facing, -1, lang.t("editor.adv_left"))
                    .changed();
                changed |= ui
                    .radio_value(facing, 1, lang.t("editor.adv_right"))
                    .changed();
            });
            changed |= ui
                .add(egui::Slider::new(walk, 0.0..=160.0).text(lang.t("editor.adv_walk")))
                .changed();
            if let Some(c) = &c {
                dialog_preview(ui, editor, lang, c, dialog);
            }
        }
        ObjectKind::Chest { contents, lock } => {
            let mut remove = None;
            for (i, (item, n)) in contents.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    changed |= combo(ui, &format!("adv_chest{i}"), "", item, &items);
                    changed |= ui.add(egui::DragValue::new(n).range(1..=999)).changed();
                    if ui.small_button("✕").clicked() {
                        remove = Some(i);
                    }
                });
            }
            if let Some(i) = remove {
                contents.remove(i);
                changed = true;
            }
            if ui.small_button(lang.t("editor.adv_add_item")).clicked() {
                contents.push(("glanztropfen".into(), 1));
                changed = true;
            }
            ui.label(lang.t("editor.adv_lock"));
            changed |= ui.text_edit_singleline(lock).changed();
        }
        ObjectKind::Switch {
            flag,
            once,
            trigger,
        } => {
            ui.horizontal(|ui| {
                ui.label(lang.t("editor.adv_flag"));
                changed |= ui.text_edit_singleline(flag).changed();
            });
            changed |= ui.checkbox(once, lang.t("editor.adv_once")).changed();
            ui.horizontal(|ui| {
                changed |= ui
                    .radio_value(
                        trigger,
                        SwitchTrigger::Interact,
                        lang.t("editor.adv_trigger_key"),
                    )
                    .changed();
                changed |= ui
                    .radio_value(
                        trigger,
                        SwitchTrigger::Hammer,
                        lang.t("editor.adv_trigger_hammer"),
                    )
                    .changed();
                changed |= ui
                    .radio_value(
                        trigger,
                        SwitchTrigger::Hook,
                        lang.t("editor.adv_trigger_hook"),
                    )
                    .changed();
            });
        }
        ObjectKind::Door { size, open_if } => {
            ui.horizontal(|ui| {
                changed |= ui
                    .add(egui::DragValue::new(&mut size.0).range(1..=32))
                    .changed();
                ui.label("×");
                changed |= ui
                    .add(egui::DragValue::new(&mut size.1).range(1..=32))
                    .changed();
                ui.label(lang.t("editor.adv_tiles"));
            });
            ui.label(lang.t("editor.adv_open_if"));
            changed |= ui.text_edit_singleline(open_if).changed();
        }
        ObjectKind::Collectible { item } => {
            changed |= combo(ui, "adv_collect", lang.t("editor.adv_item"), item, &items);
        }
        ObjectKind::HealPlant { heal } => {
            changed |= ui
                .add(egui::Slider::new(heal, 1..=10).text(lang.t("editor.adv_heal")))
                .changed();
        }
        ObjectKind::Exit {
            size,
            map,
            spawn,
            on_touch,
        } => {
            let maps = adventure_maps(editor);
            let names: Vec<String> = maps.iter().map(|(n, _)| n.clone()).collect();
            changed |= combo(
                ui,
                "adv_exit_map",
                lang.t("editor.adv_target_map"),
                map,
                &names,
            );
            let spawns: Vec<String> = maps
                .iter()
                .find(|(n, _)| n == map)
                .map(|(_, m)| {
                    m.adventure
                        .objects
                        .iter()
                        .filter(|o| matches!(o.kind, ObjectKind::Spawn))
                        .map(|o| o.id.clone())
                        .collect()
                })
                .unwrap_or_default();
            changed |= combo(
                ui,
                "adv_exit_spawn",
                lang.t("editor.adv_target_spawn"),
                spawn,
                &spawns,
            );
            changed |= ui
                .checkbox(on_touch, lang.t("editor.adv_on_touch"))
                .changed();
            size_fields(ui, lang, size, &mut changed);
        }
        ObjectKind::Zone { size } => size_fields(ui, lang, size, &mut changed),
        ObjectKind::Camera { size, mode } => {
            ui.horizontal(|ui| {
                changed |= ui
                    .radio_value(mode, CameraMode::Fixed, lang.t("editor.adv_camera_fixed"))
                    .changed();
                changed |= ui
                    .radio_value(mode, CameraMode::Bounds, lang.t("editor.adv_camera_bounds"))
                    .changed();
            });
            size_fields(ui, lang, size, &mut changed);
        }
        ObjectKind::SavePoint | ObjectKind::Spawn => {}
    }
    if changed && let Some(o) = editor.edit_object("props", now) {
        o.kind = kind;
    }
    if ui.button(lang.t("editor.adv_delete")).clicked() {
        editor.remove_object(&obj.id, now);
    }
}

fn size_fields(ui: &mut Ui, lang: &Lang, size: &mut Vec2, changed: &mut bool) {
    let ts = TILE_SIZE as f32;
    let (mut w, mut h) = ((size.x / ts).round(), (size.y / ts).round());
    ui.horizontal(|ui| {
        let a = ui
            .add(egui::DragValue::new(&mut w).range(1.0..=256.0))
            .changed();
        ui.label("×");
        let b = ui
            .add(egui::DragValue::new(&mut h).range(1.0..=256.0))
            .changed();
        ui.label(lang.t("editor.adv_tiles"));
        if a || b {
            *size = Vec2::new(w * ts, h * ts);
            *changed = true;
        }
    });
}

/// Einstiege, Knoten und Antworten eines Gesprächs; „Gespräch testen“ (E-270).
fn dialog_preview(ui: &mut Ui, editor: &mut Editor, lang: &Lang, c: &Content, dialog: &str) {
    let Some(d) = c.dialog(dialog) else {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            lang.f("editor.adv_no_dialog", &[("arg", &dialog)]),
        );
        return;
    };
    egui::CollapsingHeader::new(lang.t("editor.adv_dialog_preview"))
        .id_salt("adv_dialog_preview")
        .default_open(false)
        .show(ui, |ui| {
            for s in &d.start {
                let cond = s.cond.as_deref().unwrap_or("–");
                ui.small(format!("→ {}  [{cond}]", s.node));
            }
            for n in &d.node {
                ui.separator();
                ui.label(egui::RichText::new(format!("{}  ({})", n.id, d.speaker_of(n))).strong());
                ui.small(&n.text.de);
                if !n.actions.is_empty() {
                    ui.small(format!("do: {}", n.actions.join(", ")));
                }
                for ch in &n.choice {
                    let cond = ch
                        .cond
                        .as_deref()
                        .map(|c| format!(" [{c}]"))
                        .unwrap_or_default();
                    let next = ch.next.as_deref().unwrap_or("Ende");
                    ui.small(format!("  • {} → {next}{cond}", ch.text.de));
                }
                if let Some(next) = &n.next {
                    ui.small(format!("  → {next}"));
                }
            }
        });
    if ui.button(lang.t("editor.adv_dialog_test")).clicked() {
        let save = editor.adventure_test.save(c, &editor.adventure_id, "");
        editor.dialog_test = Some(DialogTest {
            dialog: dialog.to_owned(),
            conv: None,
            before: save.clone(),
            save,
        });
    }
}

/// Fehler der Objekte und Übergänge (Prüfung aus `elora-adventure`).
fn checks(ui: &mut Ui, editor: &Editor, lang: &Lang) {
    ui.strong(lang.t("editor.adv_checks"));
    let Some(c) = editor.content() else { return };
    let mut errors = elora_adventure::check::map_objects(c, &editor.map);
    if !editor.adventure_id.is_empty() {
        let mut maps = adventure_maps(editor);
        maps.retain(|(n, _)| *n != editor.adventure_id);
        maps.push((editor.adventure_id.clone(), editor.map.clone()));
        let refs: Vec<(&str, &Map)> = maps.iter().map(|(n, m)| (n.as_str(), m)).collect();
        errors.extend(
            elora_adventure::check::map_links(&refs)
                .into_iter()
                .filter(|e| e.contains(&format!("`{}`", editor.adventure_id))),
        );
    } else if editor.is_adventure_map() {
        errors.push(lang.t("editor.adv_map_id_missing").to_owned());
    }
    if errors.is_empty() {
        ui.small(lang.t("editor.adv_checks_ok"));
    }
    for e in errors {
        ui.colored_label(egui::Color32::LIGHT_RED, e);
    }
}

fn test_setup(ui: &mut Ui, editor: &mut Editor, lang: &Lang) {
    ui.strong(lang.t("editor.adv_test"));
    let entrances = editor.entrances();
    let t = &mut editor.adventure_test;
    ui.add(egui::Slider::new(&mut t.level, 1..=30).text(lang.t("editor.adv_level")));
    ui.horizontal_wrapped(|ui| {
        for a in Ability::ALL {
            let mut on = t.abilities.has(a);
            if ui.checkbox(&mut on, ability_name(a)).changed() {
                t.abilities.set(a, on);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut t.grenade, lang.t("adventure.weapon_grenade"));
        ui.checkbox(&mut t.laser, lang.t("adventure.weapon_laser"));
    });
    ui.horizontal(|ui| {
        ui.label(lang.t("editor.adv_flags"));
        ui.text_edit_singleline(&mut t.flags);
    });
    let current = t
        .start
        .clone()
        .unwrap_or_else(|| lang.t("editor.adv_at_mouse").to_owned());
    egui::ComboBox::from_id_salt("adv_test_start")
        .selected_text(current)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(t.start.is_none(), lang.t("editor.adv_at_mouse"))
                .clicked()
            {
                t.start = None;
            }
            for e in &entrances {
                if ui
                    .selectable_label(t.start.as_ref() == Some(e), e)
                    .clicked()
                {
                    t.start = Some(e.clone());
                }
            }
        });
    ui.small(lang.t("editor.adv_test_hint"));
}

/// Testfenster für Gespräche (E-270).
pub fn dialog_window(ctx: &egui::Context, editor: &mut Editor, lang: &Lang) {
    let Some(content) = editor.content().cloned() else {
        return;
    };
    let Some(test) = &mut editor.dialog_test else {
        return;
    };
    let mut open = true;
    egui::Window::new(lang.f("editor.adv_dialog_window", &[("arg", &test.dialog)]))
        .open(&mut open)
        .default_width(420.0)
        .show(ctx, |ui| {
            if test.conv.is_none() {
                match Conversation::start(&content, &mut test.save, &test.dialog) {
                    Some((c, turn)) if turn.open => test.conv = Some(c),
                    _ => {
                        ui.label(lang.t("editor.adv_dialog_no_start"));
                    }
                }
            }
            let mut end = false;
            if let Some(conv) = &mut test.conv
                && let Some((d, n)) = conv.current(&content)
            {
                let who = content
                    .characters
                    .get(d.speaker_of(n))
                    .map_or("Elora", |c| c.name.de.as_str());
                ui.strong(who);
                ui.label(&n.text.de);
                ui.small(&n.text.en);
                let choices = conv.choices(&content, &test.save);
                if choices.is_empty() {
                    if ui.button(lang.t("adventure.dialog_continue")).clicked() {
                        end = !conv.advance(&content, &mut test.save).open;
                    }
                } else {
                    for i in choices {
                        let label = n.choice[i].text.de.clone();
                        if ui.button(label).clicked() {
                            end = !conv.choose(&content, &mut test.save, i).open;
                            break;
                        }
                    }
                }
            }
            if end {
                test.conv = None;
                ui.label(lang.t("editor.adv_dialog_ended"));
            }
            ui.separator();
            ui.strong(lang.t("editor.adv_dialog_changes"));
            for line in changes(&test.before, &test.save) {
                ui.small(line);
            }
            if ui.button(lang.t("editor.adv_dialog_restart")).clicked() {
                test.save = test.before.clone();
                test.conv = None;
            }
        });
    if !open {
        editor.dialog_test = None;
    }
}

/// Unterschiede zwischen zwei Spielständen (Merker, Zuneigung, Aufgaben, Gegenstände).
fn changes(a: &SaveGame, b: &SaveGame) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in &b.flags {
        if a.flags.get(k) != Some(v) {
            out.push(format!("merker {k} = {v}"));
        }
    }
    for k in a.flags.keys().filter(|k| !b.flags.contains_key(*k)) {
        out.push(format!("merker {k} = 0"));
    }
    for (k, v) in &b.affection {
        if a.affection.get(k) != Some(v) {
            out.push(format!("zuneigung {k} = {v}"));
        }
    }
    for (k, v) in &b.quests {
        if a.quests.get(k) != Some(v) {
            out.push(format!("quest {k}: {:?}, Schritt {}", v.status, v.step + 1));
        }
    }
    for (k, v) in &b.inventory {
        let before = a.inventory.get(k).copied().unwrap_or(0);
        if *v != before {
            out.push(format!("{k}: {before} → {v}"));
        }
    }
    if a.glanztropfen != b.glanztropfen {
        out.push(format!(
            "glanztropfen: {} → {}",
            a.glanztropfen, b.glanztropfen
        ));
    }
    if a.level != b.level || a.xp != b.xp {
        out.push(format!("stufe {} ({} EP)", b.level, b.xp));
    }
    out
}

/// Abenteuer-Objekte auf der Karte zeichnen (Grafiken wie im Spiel, Bereiche farbig).
pub fn draw(
    batch: &mut elora_render::ShapeBatch,
    editor: &Editor,
    art: &crate::creatures::CreatureArt,
    time: f32,
) {
    use elora_render::Color;
    let z = editor.zoom;
    let area_color = |k: &ObjectKind| match k {
        ObjectKind::Exit { .. } => Color::rgba(0.35, 0.68, 0.91, 0.25),
        ObjectKind::Zone { .. } => Color::rgba(0.42, 0.75, 0.29, 0.22),
        ObjectKind::Camera { .. } => Color::rgba(0.65, 0.48, 0.88, 0.12),
        _ => Color::rgba(0.66, 0.45, 0.29, 0.5),
    };
    let selected = editor.adventure.selected.as_deref();
    for o in &editor.map.adventure.objects {
        let sel = selected == Some(o.id.as_str());
        if let Some(size) = o.kind.area() {
            let max = o.pos + size;
            batch.fill_rect(o.pos, max, area_color(&o.kind));
            let edge = if sel {
                Color::hex(0xf2c14e)
            } else {
                {
                    let mut c = area_color(&o.kind);
                    c.0[3] = 0.9;
                    c
                }
            };
            let w = if sel { 3.0 } else { 1.5 } * z;
            batch.stroke_polyline(
                &[
                    o.pos,
                    Vec2::new(max.x, o.pos.y),
                    max,
                    Vec2::new(o.pos.x, max.y),
                    o.pos,
                ],
                w,
                edge,
            );
            continue;
        }
        let h = super::adventure::height(&o.kind, |n| editor.creature_height(n));
        let ground = o.pos + Vec2::new(0.0, h.unwrap_or(0.0) / 2.0 + 1.0);
        match &o.kind {
            ObjectKind::Npc {
                character, facing, ..
            } => {
                if !art.draw_character(batch, character, ground, *facing, 1.0) {
                    batch.fill_circle(o.pos, 14.0, Color::hex(0x9aa4ae));
                }
            }
            ObjectKind::Creature { kind, persistent } => {
                let c = elora_client::scene::SceneCreature {
                    id: 0,
                    kind: kind.clone(),
                    pos: o.pos,
                    facing: -1,
                    health: 1,
                    max_health: 1,
                    since_hit: None,
                    stunned: false,
                    airborne: false,
                    boss: *persistent,
                    // ganz sichtbar (Wurzelschlange draußen)
                    mode: elora_sim::creature::burrow::OUT,
                    grow: 1.0,
                    vel: Vec2::ZERO,
                    size: Vec2::new(32.0, 32.0),
                };
                art.draw(batch, &c, time);
                if *persistent {
                    batch.stroke_circle(o.pos, 30.0, 2.0 * z, Color::hex(0xe8685a));
                }
            }
            ObjectKind::Chest { .. } => art.draw_object(batch, "truhe", ground, false),
            ObjectKind::Switch { .. } => art.draw_object(batch, "schalter", ground, false),
            ObjectKind::SavePoint => art.draw_object(batch, "quellstein", ground, true),
            ObjectKind::HealPlant { .. } => art.draw_object(batch, "heilpflanze", ground, false),
            ObjectKind::Collectible { item } => art.draw_loot_icon(batch, item, o.pos, 1.0),
            ObjectKind::Spawn => {
                batch.stroke_circle(o.pos, 13.0, 3.0, Color::hex(0xf2c14e));
                batch.fill_polygon(
                    &[
                        o.pos + Vec2::new(-5.0, -2.0),
                        o.pos + Vec2::new(5.0, -2.0),
                        o.pos + Vec2::new(0.0, 6.0),
                    ],
                    Color::hex(0xf2c14e),
                );
            }
            _ => {}
        }
        if sel {
            batch.stroke_circle(o.pos, 24.0, 2.5 * z, Color::hex(0xf2c14e));
        }
    }
}

/// Namen (Ids) der Objekte als Beschriftung über der Kartenfläche.
pub fn labels(ui: &Ui, editor: &Editor, camera: &elora_render::Camera, window: Vec2, ppp: f32) {
    if editor.zoom > 2.5 {
        return;
    }
    let painter = ui.painter();
    for o in &editor.map.adventure.objects {
        let p = o.kind.area().map_or(o.pos - Vec2::new(0.0, 30.0), |_| {
            o.pos + Vec2::new(4.0, 4.0)
        });
        let rel = p - camera.top_left();
        let px = Vec2::new(
            rel.x / camera.size.x * window.x,
            rel.y / camera.size.y * window.y,
        ) / ppp;
        let align = if o.kind.area().is_some() {
            egui::Align2::LEFT_TOP
        } else {
            egui::Align2::CENTER_BOTTOM
        };
        let text = match &o.kind {
            ObjectKind::Exit { map, spawn, .. } => format!("{} → {map}/{spawn}", o.id),
            _ => o.id.clone(),
        };
        painter.text(
            egui::pos2(px.x, px.y),
            align,
            text,
            egui::FontId::proportional(11.0),
            egui::Color32::from_white_alpha(220),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_builds_save_with_level_abilities_flags() {
        let c = Content::builtin();
        let t = TestSetup {
            level: 5,
            abilities: elora_sim::Abilities::NONE.with(Ability::Glide),
            grenade: true,
            laser: false,
            flags: "tor.dorf=2, oma.frech".into(),
            start: None,
        };
        let g = t.save(&c, "tauwinkel", "start");
        assert_eq!(g.level, 5);
        assert!(g.abilities().has(Ability::Glide));
        assert!(
            g.weapons.contains_key(&Weapon::Grenade) && !g.weapons.contains_key(&Weapon::Laser)
        );
        assert_eq!((g.flag("tor.dorf"), g.flag("oma.frech")), (2, 1));
        assert_eq!(g.health, g.max_health(&c));
    }

    /// Sichtprüfung: `cargo test -p elora-client --bin elora adventure_editor_sheet -- --ignored`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn adventure_editor_sheet() {
        use crate::editor::view;
        let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
        editor.map =
            elora_map::decode(include_bytes!("../../../../maps/abenteuer/tauwinkel.emap")).unwrap();
        editor.adventure_id = "tauwinkel".into();
        editor.tool = super::super::tools::Tool::Adventure;
        editor.adventure.selected = Some("tor".into());
        editor.center = Vec2::new(30.0 * 32.0, 14.0 * 32.0);
        editor.zoom = 1.4;
        let window = Vec2::new(1600.0, 900.0);
        let cam = view::camera(&editor, window, window * 0.5);
        let mut batch = elora_render::ShapeBatch::default();
        view::draw(
            &mut batch,
            &editor,
            &mut crate::map_view::MapView::default(),
            &crate::items::ItemArt::load(),
            &cam,
            0.5,
            Preview::None,
        );
        draw(
            &mut batch,
            &editor,
            &crate::creatures::CreatureArt::load(),
            0.5,
        );
        let tl = cam.top_left();
        let svg = batch.debug_svg(tl, tl + cam.size, view::OUTSIDE);
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/abenteuer-editor.svg"
            ),
            svg,
        )
        .unwrap();
    }
}
