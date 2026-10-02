//! Abenteuer-Modus im Spiel (R2-M1, A1.6): Spielstände, Kartenwechsel, Speichern, Tod,
//! Gespräche – über der Sandbox, die Welt, Interpolation und Szene liefert.
//!
//! Die Darstellung ist bis A1.7 schlicht (Textfeld, Hinweise im Meldungsbereich).

// Layout-Code: `s` (Skalierung), `w`/`h`/`x`/`y` wie in menu.rs
#![allow(clippy::many_single_char_names)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use elora_adventure::save::{SlotState, Slots};
use elora_adventure::session::Prompt;
use elora_adventure::state::Notice;
use elora_adventure::{Content, Conversation, Session, SessionEvent};
use elora_client::online::ChatLine;
use elora_client::scene::{ObjectLook, Scene, SceneObject};
use elora_map::{Map, ObjectKind};
use elora_render::{Align, Camera, Color};
use elora_sim::{Tuning, Vec2};

use crate::lang::{Lang, Language};
use crate::ui::{self, Rect, Ui};
use crate::{App, Screen};

/// Wie lange ein Zuruf über der Figur steht.
const BARK_SECS: f32 = 3.5;

/// Übersicht eines Platzes für das Menü.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotView {
    Empty,
    Saved {
        level: u32,
        map: String,
        play_secs: u64,
        glanz: u32,
    },
    Damaged(String),
}

/// Laufendes Abenteuer.
#[derive(Debug)]
pub struct AdventureMode {
    pub session: Session,
    pub slot: usize,
    pub conversation: Option<Conversation>,
    /// Elora ist erschöpft (E-261): verlorene Glanztropfen.
    pub dead: Option<u32>,
    barks: Vec<(String, String, Instant)>,
    /// Aktionstaste gedrückt, wird im nächsten Tick ausgewertet.
    interact: bool,
    /// Trainingskarte vor dem Abenteuer.
    previous: Option<Map>,
}

pub fn saves_dir() -> PathBuf {
    crate::settings::data_dir().map_or_else(|| PathBuf::from("saves"), |d| d.join("saves"))
}

fn slots() -> Slots {
    Slots::new(saves_dir())
}

/// Abenteuer-Karte `name` aus `maps/abenteuer/` (E-262).
fn load_map(name: &str) -> anyhow::Result<Map> {
    let path =
        elora_server::paths::resolve(Path::new("maps/abenteuer")).join(format!("{name}.emap"));
    Map::load(&path)
}

/// Übersicht aller Plätze.
pub fn slot_views() -> Vec<SlotView> {
    slots()
        .list()
        .into_iter()
        .map(|s| match s {
            SlotState::Empty => SlotView::Empty,
            SlotState::Ok(g) => SlotView::Saved {
                level: g.level,
                map: g.location.map.clone(),
                play_secs: g.play_time_secs,
                glanz: g.glanztropfen,
            },
            SlotState::Damaged(e) => SlotView::Damaged(e.to_string()),
        })
        .collect()
}

fn lang_code(l: Language) -> &'static str {
    match l {
        Language::De => "de",
        Language::En => "en",
    }
}

impl App {
    fn lang_code(&self) -> &'static str {
        lang_code(self.settings.language)
    }

    /// Abenteuer auf Platz `slot` beginnen (`new`) oder fortsetzen.
    pub(crate) fn start_adventure(&mut self, slot: usize, new: bool) {
        let content = Content::builtin();
        let session = if new {
            Session::new_game(content)
        } else {
            match slots().load(slot) {
                Ok(Some(save)) => Session::new(content, save),
                Ok(None) => Session::new_game(content),
                Err(e) => {
                    self.status = e.to_string();
                    return;
                }
            }
        };
        let loc = session.save.location.clone();
        self.adventure = Some(AdventureMode {
            session,
            slot,
            conversation: None,
            dead: None,
            barks: Vec::new(),
            interact: false,
            previous: None,
        });
        if self.travel(&loc.map, &loc.spawn) {
            if new {
                self.save_adventure();
            }
            self.enter_game();
        } else {
            self.leave_adventure();
        }
    }

    /// Karte betreten; `false`, wenn sie fehlt.
    fn travel(&mut self, map: &str, spawn: &str) -> bool {
        let Some(a) = &mut self.adventure else {
            return false;
        };
        let m = match load_map(map) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Abenteuer-Karte {map}: {e:#}");
                self.status = self.lang.f("adventure.map_missing", &[("map", &map)]);
                return false;
            }
        };
        let world = a.session.enter(map, m, spawn, &Tuning::default());
        let player = a.session.player;
        let previous = self
            .sandbox
            .play_world(a.session.map.clone(), world, player);
        a.previous.get_or_insert(previous);
        a.barks.clear();
        true
    }

    pub(crate) fn save_adventure(&mut self) {
        let Some(a) = &self.adventure else { return };
        if let Err(e) = slots().save(a.slot, &a.session.save, false) {
            self.status = self.lang.f("adventure.save_failed", &[("e", &e)]);
        }
    }

    /// Zurück ins Hauptmenü: Trainingskarte wiederherstellen. Fortschritt seit dem letzten
    /// Speichern verfällt (E-260).
    pub(crate) fn leave_adventure(&mut self) {
        if let Some(a) = self.adventure.take()
            && let Some(prev) = a.previous
        {
            self.sandbox.play_map(prev);
            self.sandbox.watch_again();
        }
        self.menu.paused = false;
        self.screen = Screen::Menu;
        self.set_cursor_grab(false);
    }

    pub(crate) fn delete_slot(&mut self, slot: usize) {
        if let Err(e) = slots().delete(slot) {
            self.status = e.to_string();
        }
    }

    /// Steht das Spiel still (Gespräch oder Erschöpfung)?
    pub(crate) fn adventure_halted(&self) -> bool {
        self.adventure
            .as_ref()
            .is_some_and(|a| a.conversation.is_some() || a.dead.is_some())
    }

    /// Aktionstaste im Spiel.
    pub(crate) fn adventure_interact(&mut self) {
        if let Some(a) = &mut self.adventure {
            if a.conversation.is_some() {
                self.dialog_continue();
                self.after_dialog();
            } else {
                a.interact = true;
            }
        }
    }

    /// Simulation und Sitzung weiterrechnen; Ereignisse auswerten.
    pub(crate) fn advance_adventure(&mut self, elapsed: Duration) {
        let Some(a) = &mut self.adventure else { return };
        if a.conversation.is_some() || a.dead.is_some() {
            return;
        }
        let mut events = Vec::new();
        let session = &mut a.session;
        let interact = &mut a.interact;
        self.sandbox
            .advance_with(elapsed, &mut self.controls, |world| {
                events.extend(session.tick(world, std::mem::take(interact)));
            });
        for e in events {
            self.adventure_event(e);
        }
    }

    fn notice(&mut self, text: String) {
        self.sandbox.notices.push_back(ChatLine {
            from: None,
            team: false,
            text,
            message: None,
            at: Instant::now(),
        });
        while self.sandbox.notices.len() > 6 {
            self.sandbox.notices.pop_front();
        }
    }

    fn notice_text(&self, n: &Notice) -> Option<String> {
        let a = self.adventure.as_ref()?;
        let c = &a.session.content;
        let code = self.lang_code();
        let quest = |id: &str| {
            c.quest(id)
                .map_or_else(|| id.to_owned(), |q| q.name.get(code).to_owned())
        };
        Some(match n {
            Notice::LevelUp { level } => self.lang.f("adventure.level_up", &[("n", level)]),
            Notice::Xp(_) => return None,
            Notice::Item { id, count } => {
                let name = c.item(id).map_or(id.as_str(), |i| i.name.get(code));
                self.lang
                    .f("adventure.item", &[("n", count), ("item", &name)])
            }
            Notice::QuestStarted(id) => self
                .lang
                .f("adventure.quest_started", &[("name", &quest(id))]),
            Notice::QuestStep(id) => self.lang.f("adventure.quest_step", &[("name", &quest(id))]),
            Notice::QuestDone(id) => self.lang.f("adventure.quest_done", &[("name", &quest(id))]),
            Notice::QuestFailed(id) => self
                .lang
                .f("adventure.quest_failed", &[("name", &quest(id))]),
        })
    }

    fn adventure_event(&mut self, e: SessionEvent) {
        match e {
            SessionEvent::Notice(n) => {
                if let Some(t) = self.notice_text(&n) {
                    self.notice(t);
                }
            }
            SessionEvent::Talk { dialog, .. } => self.start_dialog(&dialog),
            SessionEvent::Bark { npc, text } => {
                let code = self.lang_code();
                if let Some(a) = &mut self.adventure {
                    let until = Instant::now() + Duration::from_secs_f32(BARK_SECS);
                    a.barks.retain(|b| b.0 != npc);
                    a.barks.push((npc, text.get(code).to_owned(), until));
                }
            }
            SessionEvent::Locked { .. } => {
                let t = self.lang.t("adventure.locked").to_owned();
                self.notice(t);
            }
            SessionEvent::Travel { map, spawn } => {
                if self.travel(&map, &spawn) {
                    self.save_adventure();
                }
            }
            SessionEvent::Save => {
                self.save_adventure();
                let t = self.lang.t("adventure.saved").to_owned();
                self.notice(t);
            }
            SessionEvent::Died { lost } => {
                if let Some(a) = &mut self.adventure {
                    a.dead = Some(lost);
                }
                self.controls.release_all();
                self.set_cursor_grab(false);
            }
            SessionEvent::TilesChanged => {
                if let Some(a) = &self.adventure {
                    self.sandbox.map.tiles.clone_from(&a.session.map.tiles);
                }
            }
            // Laden, Schmiede und Baum kommen mit der Oberfläche (A1.7)
            SessionEvent::Open(_) => {}
        }
    }

    /// Nach der Erschöpfung am letzten Speicherpunkt weiter (E-220, E-261).
    pub(crate) fn adventure_respawn(&mut self) {
        let Some(a) = &mut self.adventure else { return };
        a.dead = None;
        let loc = a.session.save.location.clone();
        if self.travel(&loc.map, &loc.spawn) {
            self.save_adventure();
            self.set_cursor_grab(true);
        }
    }

    // ------------------------------------------------------------ Gespräche

    fn start_dialog(&mut self, dialog: &str) {
        let Some(a) = &mut self.adventure else { return };
        let s = &mut a.session;
        let Some((conv, turn)) = Conversation::start(&s.content, &mut s.save, dialog) else {
            return;
        };
        a.conversation = turn.open.then_some(conv);
        self.controls.release_all();
        if self
            .adventure
            .as_ref()
            .is_some_and(|a| a.conversation.is_some())
        {
            self.set_cursor_grab(false);
        }
        for o in turn.outcomes {
            self.adventure_event(outcome_event(o));
        }
    }

    /// Tasten während Gespräch oder Erschöpfung: Ziffern wählen, E/Leertaste/Enter weiter.
    pub(crate) fn adventure_key(&mut self, code: winit::keyboard::KeyCode) {
        use winit::keyboard::KeyCode as K;
        let digits = [
            K::Digit1,
            K::Digit2,
            K::Digit3,
            K::Digit4,
            K::Digit5,
            K::Digit6,
        ];
        if let Some(n) = digits.iter().position(|d| *d == code) {
            self.dialog_choose(n);
        } else if matches!(code, K::KeyE | K::Space | K::Enter) {
            self.dialog_continue();
        }
        self.after_dialog();
    }

    /// Nach dem Gespräch die Maus wieder fangen.
    fn after_dialog(&mut self) {
        if self
            .adventure
            .as_ref()
            .is_some_and(|a| a.conversation.is_none() && a.dead.is_none())
        {
            self.set_cursor_grab(true);
        }
    }

    /// Weiter ohne Auswahl (Knoten ohne Antworten).
    pub(crate) fn dialog_continue(&mut self) {
        let Some(a) = &mut self.adventure else { return };
        let Some(conv) = &mut a.conversation else {
            return;
        };
        let s = &mut a.session;
        if !conv.choices(&s.content, &s.save).is_empty() {
            return;
        }
        let turn = conv.advance(&s.content, &mut s.save);
        if !turn.open {
            a.conversation = None;
        }
        for o in turn.outcomes {
            self.adventure_event(outcome_event(o));
        }
    }

    /// Antwort Nummer `n` (ab 0) der sichtbaren Antworten.
    pub(crate) fn dialog_choose(&mut self, n: usize) {
        let Some(a) = &mut self.adventure else { return };
        let Some(conv) = &mut a.conversation else {
            return;
        };
        let s = &mut a.session;
        let Some(&index) = conv.choices(&s.content, &s.save).get(n) else {
            return;
        };
        let turn = conv.choose(&s.content, &mut s.save, index);
        if !turn.open {
            a.conversation = None;
        }
        for o in turn.outcomes {
            self.adventure_event(outcome_event(o));
        }
    }

    // ------------------------------------------------------------ Darstellung

    /// NPCs und Objekte in die Szene.
    pub(crate) fn adventure_scene(&self, scene: &mut Scene) {
        let Some(a) = &self.adventure else { return };
        let s = &a.session;
        for n in s.npcs(&self.sandbox.world) {
            scene.objects.push(SceneObject {
                look: ObjectLook::Npc {
                    character: n.character,
                    facing: n.facing,
                },
                pos: n.pos,
            });
        }
        for o in &s.map.adventure.objects {
            let done = s.object_done(&o.id);
            let look = match &o.kind {
                ObjectKind::Chest { .. } => ObjectLook::Chest { open: done },
                ObjectKind::Switch { .. } => ObjectLook::Switch { on: done },
                ObjectKind::SavePoint => ObjectLook::SavePoint,
                ObjectKind::HealPlant { .. } => ObjectLook::HealPlant { used: done },
                ObjectKind::Collectible { item } if !done => {
                    ObjectLook::Collectible { item: item.clone() }
                }
                _ => continue,
            };
            scene.objects.push(SceneObject { look, pos: o.pos });
        }
    }

    /// Hinweis zur Aktionstaste, Zurufe, Gesprächsfeld und Erschöpfung in den HUD-Batch.
    /// Liefert `true`, wenn „Weiter am Quellstein“ bzw. `false` für „Hauptmenü“ gewählt wurde.
    #[allow(clippy::too_many_lines)] // Zeichenreihenfolge an einem Ort, bis A1.7 aufteilt
    pub(crate) fn draw_adventure_hud(
        &mut self,
        camera: &Camera,
        screen: Vec2,
        s: f32,
        dt: f32,
    ) -> Option<bool> {
        let code = self.lang_code();
        let Some(a) = &mut self.adventure else {
            return None;
        };
        let now = Instant::now();
        a.barks.retain(|b| b.2 > now);
        let to_screen = |p: Vec2| {
            let rel = p - camera.top_left();
            Vec2::new(
                rel.x / camera.size.x * screen.x,
                rel.y / camera.size.y * screen.y,
            )
        };
        let world = &self.sandbox.world;
        let session = &a.session;
        let mut ui = Ui {
            batch: &mut self.hud_batch,
            font: self.hud.font(),
            input: &self.menu.input,
            state: &mut self.menu.ui,
            s,
        };
        ui.begin(dt);
        // Zurufe über den Figuren
        let npcs = session.npcs(world);
        for (npc, text, _) in &a.barks {
            if let Some(n) = npcs.iter().find(|n| &n.id == npc) {
                let p = to_screen(n.pos - Vec2::new(0.0, 46.0));
                bubble(&mut ui, text, p, s);
            }
        }
        // Hinweis zur Aktionstaste
        if a.conversation.is_none()
            && a.dead.is_none()
            && let Some(ch) = world.character(session.player)
            && let Some((id, prompt)) = session.interactable(ch.core.pos)
            && let Some(o) = session.map.adventure.object(&id)
        {
            let key = match prompt {
                Prompt::Talk => "adventure.prompt_talk",
                Prompt::Open => "adventure.prompt_open",
                Prompt::Use => "adventure.prompt_use",
                Prompt::Enter => "adventure.prompt_enter",
                Prompt::Rest => "adventure.prompt_rest",
            };
            let anchor = o
                .kind
                .area()
                .map_or(o.pos, |sz| o.pos + Vec2::new(sz.x / 2.0, 0.0));
            let p = to_screen(anchor - Vec2::new(0.0, 40.0));
            bubble(&mut ui, &format!("E  {}", self.lang.t(key)), p, s);
        }
        // Gespräch (schlichte Fassung bis A1.7)
        let mut chosen = None;
        if let Some(conv) = &a.conversation
            && let Some((d, node)) = conv.current(&session.content)
        {
            let w = (screen.x - 80.0 * s).min(900.0 * s);
            let choices = conv.choices(&session.content, &session.save);
            #[allow(clippy::cast_precision_loss)]
            let h = (110.0 + choices.len() as f32 * 30.0) * s;
            let card = Rect::new((screen.x - w) / 2.0, screen.y - h - 24.0 * s, w, h);
            ui.card(card);
            let speaker = d.speaker_of(node);
            let name = session
                .content
                .characters
                .get(speaker)
                .map_or("Elora", |c| c.name.get(code));
            ui.label(
                name,
                card.min + Vec2::new(20.0 * s, 24.0 * s),
                14.0,
                ui::VIOLET,
                Align::Left,
            );
            let mut y = card.min.y + 50.0 * s;
            for line in wrap(&ui, node.text.get(code), 13.0, w - 40.0 * s) {
                ui.label(
                    &line,
                    Vec2::new(card.min.x + 20.0 * s, y),
                    13.0,
                    ui::TEXT,
                    Align::Left,
                );
                y += 19.0 * s;
            }
            if choices.is_empty() {
                ui.label(
                    &format!("E  {}", self.lang.t("adventure.dialog_continue")),
                    Vec2::new(card.max.x - 20.0 * s, card.max.y - 16.0 * s),
                    11.0,
                    ui::TEXT_DIM,
                    Align::Right,
                );
            }
            for (k, &i) in choices.iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let r = Rect::new(
                    card.min.x + 16.0 * s,
                    card.max.y - (choices.len() - k) as f32 * 30.0 * s - 10.0 * s,
                    w - 32.0 * s,
                    26.0 * s,
                );
                let text = format!("{}  {}", k + 1, node.choice[i].text.get(code));
                if ui.button(&format!("dlg{k}"), r, &text, ui::SAND) {
                    chosen = Some(k);
                }
            }
        }
        // Erschöpft (E-261)
        let mut death = None;
        if let Some(lost) = a.dead {
            ui.batch
                .fill_rect(Vec2::ZERO, screen, Color::rgba(0.9, 0.92, 0.95, 0.55));
            let (w, h) = (420.0 * s, 200.0 * s);
            let card = Rect::new((screen.x - w) / 2.0, (screen.y - h) / 2.0, w, h);
            ui.card(card);
            ui.label(
                self.lang.t("adventure.died"),
                Vec2::new(card.center().x, card.min.y + 40.0 * s),
                20.0,
                ui::TEXT,
                Align::Center,
            );
            if lost > 0 {
                ui.label(
                    &self.lang.f("adventure.lost", &[("n", &lost)]),
                    Vec2::new(card.center().x, card.min.y + 72.0 * s),
                    13.0,
                    ui::TEXT_DIM,
                    Align::Center,
                );
            }
            let b = Rect::new(
                card.min.x + 24.0 * s,
                card.max.y - 70.0 * s,
                w - 48.0 * s,
                30.0 * s,
            );
            if ui.button(
                "adv_respawn",
                b,
                self.lang.t("adventure.respawn"),
                ui::GREEN,
            ) {
                death = Some(true);
            }
            let b = b.offset(Vec2::new(0.0, 36.0 * s));
            if ui.button(
                "adv_menu",
                Rect::new(b.min.x, b.min.y, b.w(), 26.0 * s),
                self.lang.t("adventure.to_menu"),
                ui::GRAY,
            ) {
                death = Some(false);
            }
        }
        ui.end();
        self.menu.input.next_frame();
        if let Some(k) = chosen {
            self.dialog_choose(k);
            self.after_dialog();
        }
        death
    }
}

fn outcome_event(o: elora_adventure::Outcome) -> SessionEvent {
    match o {
        elora_adventure::Outcome::Notice(n) => SessionEvent::Notice(n),
        elora_adventure::Outcome::Open(x) => SessionEvent::Open(x),
    }
}

/// Sprechblase mit Text, unten mittig an `p`.
fn bubble(ui: &mut Ui<'_>, text: &str, p: Vec2, s: f32) {
    let w = ui.text_width(text, 12.0) + 20.0 * s;
    let r = Rect::new(p.x - w / 2.0, p.y - 26.0 * s, w, 24.0 * s);
    ui.batch.fill_rounded_rect(
        r.min - Vec2::new(1.5 * s, 1.5 * s),
        r.max + Vec2::new(1.5 * s, 1.5 * s),
        12.0 * s,
        ui::OUTLINE,
    );
    ui.batch
        .fill_rounded_rect(r.min, r.max, 11.0 * s, ui::FIELD);
    ui.label(text, r.center(), 12.0, ui::TEXT, Align::Center);
}

/// Zeilenumbruch an Wortgrenzen für die Breite `max` (Pixel).
fn wrap(ui: &Ui<'_>, text: &str, size: f32, max: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if ui.text_width(&next, size) > max && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Text zur Zeit `secs` (Menü).
pub fn play_time(lang: &Lang, secs: u64) -> String {
    lang.f(
        "adventure.time",
        &[("h", &(secs / 3600)), ("m", &((secs / 60) % 60))],
    )
}
