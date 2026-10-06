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
/// Vorausschau der Kamera in die zuletzt gelaufene Richtung (E-266, E-267): 3 Tiles, bleibt beim
/// Stehenbleiben stehen und wechselt erst nach kurzem Laufen in die andere Richtung.
const LOOKAHEAD: f32 = 96.0;
/// Gesprächstext: Zeichen je Sekunde (E-286).
const REVEAL_PER_SECOND: f32 = 48.0;
/// Ein Plapperlaut je so viele Zeichen.
const SYLLABLE: usize = 3;
/// Tonhöhe von Eloras Stimme.
const ELORA_VOICE: f32 = 1.25;
/// Ab diesem Tempo (Einheiten/Tick) zählt Laufen als Richtung.
const LOOKAHEAD_MIN_SPEED: f32 = 2.0;
/// So lange (s) muss Elora in die andere Richtung laufen, bis die Kamera wechselt.
const LOOKAHEAD_TURN_SECS: f32 = 0.4;
/// Nachziehen je Sekunde (Vorausschau, Übergang in Kamera-Zonen).
const LOOKAHEAD_RATE: f32 = 1.5;
const ZONE_RATE: f32 = 3.0;

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
    /// Spielstand-Platz; `None` beim Testspiel aus dem Editor (es wird nicht gespeichert, E-269).
    pub slot: Option<usize>,
    /// Testspiel aus dem Editor: diese Karte (ungespeichert) statt der Datei gleichen Namens.
    editor_map: Option<(String, Map)>,
    pub conversation: Option<Conversation>,
    /// Elora ist erschöpft (E-261): verlorene Glanztropfen.
    pub dead: Option<u32>,
    /// Hüter beruhigt: Gewinn-Bildschirm für dieses Gebiet, seit diesem Zeitpunkt.
    pub victory: Option<(String, Instant)>,
    barks: Vec<(String, String, Instant)>,
    /// Aktionstaste gedrückt, wird im nächsten Tick ausgewertet.
    interact: bool,
    /// Trainingskarte vor dem Abenteuer.
    previous: Option<Map>,
    /// Vorausschau der Kamera (Einheiten) und Anteil der Kamera-Zone (0..1, weicher Übergang).
    look: f32,
    /// Richtung der Vorausschau (−1, 0, 1) und Zeit in der Gegenrichtung (E-267).
    look_dir: i8,
    look_turn: f32,
    zone_mix: f32,
    zone_center: Vec2,
    /// Abenteuer-Menü offen (Tab, Laden, Schmiede, E-263).
    pub menu: Option<crate::adventure_menu::MenuState>,
    /// Auswahl im Menü bleibt zwischen dem Öffnen erhalten.
    menu_memory: crate::adventure_menu::MenuState,
    /// Gesprächstext erscheint nach und nach (E-286): angezeigte Zeichen, Gesamtlänge, Knoten.
    reveal: f32,
    reveal_total: usize,
    reveal_key: (String, String),
}

pub fn saves_dir() -> PathBuf {
    crate::settings::data_dir().map_or_else(|| PathBuf::from("saves"), |d| d.join("saves"))
}

fn slots() -> Slots {
    Slots::new(saves_dir())
}

/// Abenteuer-Karte `name`: zuerst aus dem Benutzerordner `maps/abenteuer/` (E-271), sonst die
/// mitgelieferte (E-262).
fn load_map(name: &str) -> anyhow::Result<Map> {
    let file = format!("{name}.emap");
    if let Some(own) = crate::settings::user_maps_dir().map(|d| d.join("abenteuer").join(&file))
        && own.is_file()
    {
        return Map::load(&own);
    }
    Map::load(&elora_server::paths::resolve(Path::new("maps/abenteuer")).join(file))
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
            slot: Some(slot),
            editor_map: None,
            conversation: None,
            dead: None,
            victory: None,
            barks: Vec::new(),
            interact: false,
            previous: None,
            look: 0.0,
            look_dir: 0,
            look_turn: 0.0,
            zone_mix: 0.0,
            zone_center: Vec2::ZERO,
            menu: None,
            menu_memory: crate::adventure_menu::MenuState::default(),
            reveal: 0.0,
            reveal_total: 0,
            reveal_key: (String::new(), String::new()),
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
        let editor_map = a
            .editor_map
            .as_ref()
            .filter(|(n, _)| n == map)
            .map(|(_, m)| Ok(m.clone()));
        let m = match editor_map.unwrap_or_else(|| load_map(map)) {
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
        let Some(slot) = a.slot else { return };
        if let Err(e) = slots().save(slot, &a.session.save, false) {
            self.status = self.lang.f("adventure.save_failed", &[("e", &e)]);
        }
    }

    /// Zurück ins Hauptmenü: Trainingskarte wiederherstellen. Fortschritt seit dem letzten
    /// Speichern verfällt (E-260).
    pub(crate) fn leave_adventure(&mut self) {
        let from_editor = self.adventure.as_ref().is_some_and(|a| a.slot.is_none());
        if let Some(a) = self.adventure.take()
            && let Some(prev) = a.previous
        {
            self.sandbox.play_map(prev);
            self.sandbox.watch_again();
        }
        self.menu.paused = false;
        self.screen = if from_editor {
            Screen::Editor
        } else {
            Screen::Menu
        };
        self.set_cursor_grab(false);
    }

    /// Testspiel aus dem Editor (E-269): Teststand, ungespeicherte Karte, kein Spielstand.
    pub(crate) fn start_adventure_test(&mut self) {
        let Some(editor) = &self.editor else { return };
        let content = editor.content().cloned().unwrap_or_else(Content::builtin);
        let id = if editor.adventure_id.is_empty() {
            "editor-test".to_owned()
        } else {
            editor.adventure_id.clone()
        };
        let mut map = editor.map.clone();
        let spawn = if let Some(s) = &editor.adventure_test.start {
            s.clone()
        } else {
            // an der Maus: vorübergehender Eingang
            let pos = editor.mouse_world.unwrap_or(Vec2::new(64.0, 64.0));
            map.adventure.objects.push(elora_map::Object {
                id: "editor-maus".into(),
                pos,
                kind: ObjectKind::Spawn,
            });
            "editor-maus".to_owned()
        };
        let save = editor.adventure_test.save(&content, &id, &spawn);
        self.online = None;
        self.adventure = Some(AdventureMode {
            session: Session::new(content, save),
            slot: None,
            editor_map: Some((id.clone(), map)),
            conversation: None,
            dead: None,
            victory: None,
            barks: Vec::new(),
            interact: false,
            previous: None,
            look: 0.0,
            look_dir: 0,
            look_turn: 0.0,
            zone_mix: 0.0,
            zone_center: Vec2::ZERO,
            menu: None,
            menu_memory: crate::adventure_menu::MenuState::default(),
            reveal: 0.0,
            reveal_total: 0,
            reveal_key: (String::new(), String::new()),
        });
        if self.travel(&id, &spawn) {
            self.enter_game();
        } else {
            self.leave_adventure();
        }
    }

    /// Läuft ein Testspiel aus dem Editor?
    pub(crate) fn testing_adventure(&self) -> bool {
        self.adventure.as_ref().is_some_and(|a| a.slot.is_none())
    }

    pub(crate) fn delete_slot(&mut self, slot: usize) {
        if let Err(e) = slots().delete(slot) {
            self.status = e.to_string();
        }
    }

    /// Steht das Spiel still (Gespräch oder Erschöpfung)?
    pub(crate) fn adventure_halted(&self) -> bool {
        self.adventure.as_ref().is_some_and(|a| {
            a.conversation.is_some() || a.dead.is_some() || a.menu.is_some() || a.victory.is_some()
        })
    }

    /// Gewinn-Bildschirm schließen (nach [`crate::adventure_hud::VICTORY_READY`]).
    fn close_victory(&mut self) {
        let Some(a) = &mut self.adventure else { return };
        let ready = a.victory.as_ref().is_some_and(|(_, at)| {
            at.elapsed().as_secs_f32() >= crate::adventure_hud::VICTORY_READY
        });
        if ready {
            a.victory = None;
            self.ui_cues
                .push(elora_audio::Cue::global(elora_audio::Sound::UiClose));
            self.set_cursor_grab(true);
        }
    }

    /// Abenteuer-Menü öffnen (`panel`) oder schließen (`None`).
    pub(crate) fn adventure_menu(&mut self, panel: Option<crate::adventure_menu::Panel>) {
        let Some(a) = &mut self.adventure else { return };
        if let Some(p) = panel {
            if a.menu.is_none() {
                self.ui_cues
                    .push(elora_audio::Cue::global(elora_audio::Sound::UiOpen));
            }
            let mut st = a.menu.take().unwrap_or_else(|| a.menu_memory.clone());
            st.panel = p;
            st.refusal = None;
            a.menu = Some(st);
            self.controls.release_all();
            self.set_cursor_grab(false);
        } else {
            if let Some(st) = a.menu.take() {
                self.ui_cues
                    .push(elora_audio::Cue::global(elora_audio::Sound::UiClose));
                // Laden und Schmiede merken sich nicht als letzte Seite
                if !matches!(
                    st.panel,
                    crate::adventure_menu::Panel::Shop(_) | crate::adventure_menu::Panel::Forge
                ) {
                    a.menu_memory = st;
                }
            }
            self.after_dialog();
        }
    }

    /// Tab: Abenteuer-Menü auf/zu (E-263).
    pub(crate) fn toggle_adventure_menu(&mut self) {
        let Some(a) = &self.adventure else { return };
        if a.conversation.is_some() || a.dead.is_some() {
            return;
        }
        if a.menu.is_some() {
            self.adventure_menu(None);
        } else {
            let p = a.menu_memory.panel.clone();
            self.adventure_menu(Some(p));
        }
    }

    /// Q: Heiltrank trinken (E-265).
    pub(crate) fn quick_heal(&mut self) {
        let Some(a) = &mut self.adventure else { return };
        if a.session.save.count("heiltrank") == 0 {
            return;
        }
        let full = self
            .sandbox
            .world
            .character(a.session.player)
            .is_none_or(|c| c.health >= a.session.save.max_health(&a.session.content));
        if !full {
            let _ = a.session.use_item(&mut self.sandbox.world, "heiltrank");
        }
    }

    /// Befehl aus dem Abenteuer-Menü ausführen.
    fn adventure_command(&mut self, cmd: crate::adventure_menu::Command) {
        use crate::adventure_menu::Command as C;
        let Some(a) = &mut self.adventure else { return };
        let world = &mut self.sandbox.world;
        let s = &mut a.session;
        let c = s.content.clone();
        let result = match cmd {
            C::Close => {
                self.adventure_menu(None);
                return;
            }
            C::Equip(id) => s.save.equip(&c, &id),
            C::Unequip(slot) => {
                s.save.unequip(&c, slot);
                Ok(())
            }
            C::Use(id) => s.use_item(world, &id),
            C::Learn(id) => s.save.learn(&c, &id).map(|_| ()),
            C::Buy(shop, id) => s.save.buy(&c, &shop, &id),
            C::Sell(id) => s.save.sell(&c, &id).map(|_| ()),
            C::Upgrade(w) => s.save.upgrade(&c, w).map(|_| ()),
        };
        if let Some(m) = &mut a.menu {
            m.refusal = result.err();
        }
        // Werte aus Baum, Ausrüstung und Ausbau sofort in die Welt (Tuning, Leben, Rüstung)
        let base = elora_sim::Tuning::default();
        world.tuning = s.save.tuning(&c, &base);
        let max = s.save.max_health(&c);
        let armor = s.save.stats(&c).armor.max(0);
        if let Some(ch) = world.character_mut(s.player) {
            ch.health = ch.health.min(max);
            // neue Rüstung füllt sich erst am Quellstein (P-23)
            ch.armor = ch.armor.min(armor);
        }
        s.save.health = s.save.health.min(max);
        s.sync_world(world);
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
        if a.conversation.is_some() || a.dead.is_some() || a.victory.is_some() {
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
                // Klang zu Funden und erledigten Aufgaben (R2-M2.1)
                use elora_adventure::data::ItemKind;
                use elora_audio::{Cue, Sound};
                let sound = match &n {
                    Notice::QuestDone(_) => Some(Sound::QuestDone),
                    Notice::Item { id, .. } => self.adventure.as_ref().and_then(|a| {
                        match a.session.content.item(id).map(|i| &i.kind) {
                            Some(ItemKind::Key) => Some(Sound::Fanfare),
                            Some(ItemKind::Collectible) => Some(Sound::Collect),
                            _ => None,
                        }
                    }),
                    _ => None,
                };
                self.ui_cues.extend(sound.map(Cue::global));
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
                    let line = crate::adventure_hud::with_keys(
                        text.get(code),
                        &self.settings.bindings,
                        &self.lang,
                    );
                    a.barks.push((npc, line, until));
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
            SessionEvent::ChapterDone { area } => {
                if let Some(a) = &mut self.adventure {
                    a.victory = Some((area, Instant::now()));
                }
                self.ui_cues
                    .push(elora_audio::Cue::global(elora_audio::Sound::Fanfare));
                self.controls.release_all();
                self.set_cursor_grab(false);
            }
            SessionEvent::TilesChanged => {
                if let Some(a) = &self.adventure {
                    self.sandbox.map.tiles.clone_from(&a.session.map.tiles);
                }
            }
            SessionEvent::Open(o) => {
                use crate::adventure_menu::Panel;
                use elora_adventure::script::Open;
                let panel = match o {
                    Open::Shop(id) => Panel::Shop(id),
                    Open::Forge => Panel::Forge,
                    Open::Skills => Panel::Skills,
                };
                // nach dem Gespräch öffnen
                if let Some(a) = &mut self.adventure {
                    a.conversation = None;
                }
                self.adventure_menu(Some(panel));
            }
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
        // Gewinn-Bildschirm: E, Leertaste oder Enter führt weiter
        if self.adventure.as_ref().is_some_and(|a| a.victory.is_some()) {
            if matches!(code, K::KeyE | K::Space | K::Enter | K::Escape) {
                self.close_victory();
            }
            return;
        }
        // Abenteuer-Menü, Laden, Schmiede: Tab (belegte Taste) oder Esc schließt
        if self.adventure.as_ref().is_some_and(|a| a.menu.is_some()) {
            let tab = self
                .settings
                .bindings
                .trigger(crate::bindings::GameAction::Scoreboard);
            if code == K::Escape || crate::bindings::Trigger::Key(code) == tab {
                self.adventure_menu(None);
            }
            return;
        }
        let digits = [
            K::Digit1,
            K::Digit2,
            K::Digit3,
            K::Digit4,
            K::Digit5,
            K::Digit6,
        ];
        if let Some(n) = digits.iter().position(|d| *d == code) {
            self.ui_cues
                .push(elora_audio::Cue::global(elora_audio::Sound::UiClick));
            self.dialog_choose(n);
        } else if matches!(code, K::KeyE | K::Space | K::Enter) {
            self.dialog_continue();
        }
        self.after_dialog();
    }

    /// Nach dem Gespräch die Maus wieder fangen.
    fn after_dialog(&mut self) {
        // Gespräche schalten frei (Hook-Ruck, Waffen): sofort in die Welt
        if let Some(a) = &mut self.adventure {
            a.session.sync_world(&mut self.sandbox.world);
        }
        // Gespräche setzen Merker: Deko nachziehen (Quelle blüht, Festschmuck)
        if let Some(a) = &mut self.adventure
            && a.session.refresh_decor()
        {
            self.sandbox
                .map
                .decor_back
                .clone_from(&a.session.map.decor_back);
            self.sandbox
                .map
                .decor_front
                .clone_from(&a.session.map.decor_front);
            // Quelle befreit: das Gebiet klart auf (R2-W1)
            self.sandbox.map.weather = a.session.map.weather;
        }
        if self
            .adventure
            .as_ref()
            .is_some_and(|a| a.conversation.is_none() && a.dead.is_none() && a.menu.is_none())
        {
            self.set_cursor_grab(true);
        }
    }

    /// Text noch nicht ganz da: auf einmal zeigen (`true`, dann nichts weiter tun).
    fn reveal_rest(&mut self) -> bool {
        let Some(a) = &mut self.adventure else {
            return false;
        };
        #[allow(clippy::cast_precision_loss)]
        let total = a.reveal_total as f32;
        if a.conversation.is_some() && a.reveal < total {
            a.reveal = total;
            return true;
        }
        false
    }

    /// Weiter ohne Auswahl (Knoten ohne Antworten).
    pub(crate) fn dialog_continue(&mut self) {
        if self.reveal_rest() {
            return;
        }
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
        if self.reveal_rest() {
            return;
        }
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

    /// Kameramitte im Abenteuer (E-224, E-259, E-266): `center` folgt Elora, dazu Vorausschau
    /// in Laufrichtung; Kamera-Zonen setzen den Ausschnitt fest oder begrenzen ihn.
    pub(crate) fn adventure_camera(&mut self, center: Vec2, view: Vec2, dt: f32) -> Vec2 {
        use elora_map::adventure::CameraMode;
        let Some(a) = &mut self.adventure else {
            return center;
        };
        let vel = self
            .sandbox
            .world
            .character(a.session.player)
            .map_or(Vec2::ZERO, |c| c.core.vel);
        if a.conversation.is_none() && a.menu.is_none() {
            if vel.x.abs() > LOOKAHEAD_MIN_SPEED {
                let dir: i8 = if vel.x < 0.0 { -1 } else { 1 };
                if a.look_dir == 0 {
                    a.look_dir = dir;
                } else if dir == a.look_dir {
                    a.look_turn = 0.0;
                } else {
                    a.look_turn += dt;
                    if a.look_turn > LOOKAHEAD_TURN_SECS {
                        a.look_dir = dir;
                        a.look_turn = 0.0;
                    }
                }
            }
            // bei Nebel und Stürmen schaut die Kamera weniger weit voraus (R2-W1, E-330)
            let w = a.session.map.weather;
            let sight = match w.kind {
                elora_map::WeatherKind::Fog
                | elora_map::WeatherKind::Sandstorm
                | elora_map::WeatherKind::Blizzard => 1.0 - 0.5 * w.intensity,
                elora_map::WeatherKind::Storm => 1.0 - 0.2 * w.intensity,
                _ => 1.0,
            };
            let target = f32::from(a.look_dir) * LOOKAHEAD * sight;
            a.look += (target - a.look) * (dt * LOOKAHEAD_RATE).min(1.0);
        }
        let free = center + Vec2::new(a.look, 0.0);
        let zone = a
            .session
            .map
            .adventure
            .objects
            .iter()
            .find_map(|o| match &o.kind {
                ObjectKind::Camera { size, mode }
                    if center.x >= o.pos.x
                        && center.y >= o.pos.y
                        && center.x <= o.pos.x + size.x
                        && center.y <= o.pos.y + size.y =>
                {
                    Some(match mode {
                        CameraMode::Fixed => o.pos + *size * 0.5,
                        CameraMode::Bounds => {
                            // Ausschnitt bleibt im Bereich; ist er kleiner als die Sicht, mittig
                            let clamp = |v: f32, min: f32, len: f32, half: f32| {
                                if len <= half * 2.0 {
                                    min + len / 2.0
                                } else {
                                    v.clamp(min + half, min + len - half)
                                }
                            };
                            Vec2::new(
                                clamp(free.x, o.pos.x, size.x, view.x / 2.0),
                                clamp(free.y, o.pos.y, size.y, view.y / 2.0),
                            )
                        }
                    })
                }
                _ => None,
            });
        let goal = if zone.is_some() { 1.0 } else { 0.0 };
        if let Some(z) = zone {
            a.zone_center = z;
        }
        a.zone_mix += (goal - a.zone_mix) * (dt * ZONE_RATE).min(1.0);
        free.lerp(a.zone_center, a.zone_mix.clamp(0.0, 1.0))
    }

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
                ObjectKind::SavePoint => ObjectLook::SavePoint {
                    active: s.save.location.map == s.map_name && s.save.location.spawn == o.id,
                },
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
        let content = &session.content;
        #[allow(clippy::cast_precision_loss)]
        let secs = world.tick as f32 / elora_sim::TICKS_PER_SECOND as f32;
        let hot = session.hot();
        if a.conversation.is_none() && a.dead.is_none() && a.menu.is_none() {
            crate::adventure_hud::status(
                &mut ui,
                &self.creature_art,
                &self.lang,
                code,
                content,
                &session.save,
                screen,
            );
            if hot {
                crate::adventure_hud::heat_bar(&mut ui, session.heat, session.overheated, secs);
            }
        }
        // Lebensleiste eines wachen Hüters (R2-M2.1)
        if a.conversation.is_none()
            && a.menu.is_none()
            && let Some((c, k)) = world
                .creatures
                .iter()
                .map(|c| (c, &world.creature_kinds[c.kind]))
                .find(|(c, k)| k.boss && c.mode != elora_sim::creature::diver::SLEEP)
        {
            #[allow(clippy::cast_precision_loss)]
            let frac = c.health as f32 / k.health.max(1) as f32;
            let name = self.lang.t(&format!("creature.{}", k.name)).to_owned();
            crate::adventure_hud::boss_bar(&mut ui, &name, frac, screen);
        }
        // Zurufe über den Figuren
        let npcs = session.npcs(world);
        for (npc, text, _) in &a.barks {
            if let Some(n) = npcs.iter().find(|n| &n.id == npc) {
                let p = to_screen(n.pos - Vec2::new(0.0, 44.0));
                crate::adventure_hud::bubble(&mut ui, text, p);
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
            let anchor = session.anchor(&id).unwrap_or(o.pos);
            let p = to_screen(anchor - Vec2::new(0.0, 40.0));
            let label = self
                .settings
                .bindings
                .trigger(crate::bindings::GameAction::Interact)
                .label(&self.lang);
            crate::adventure_hud::prompt(&mut ui, &label, self.lang.t(key), p);
            // Name der Figur über dem Hinweis
            if let ObjectKind::Npc { character, .. } = &o.kind
                && let Some(c) = content.characters.get(character)
            {
                let above = p - Vec2::new(0.0, 34.0 * s);
                crate::adventure_hud::name_tag(&mut ui, c.name.get(code), character, above);
            }
        }
        // Gespräch (E-222)
        let mut chosen = None;
        if let Some(conv) = &a.conversation
            && let Some((d, node)) = conv.current(content)
        {
            let speaker = d.speaker_of(node);
            // Text erscheint nach und nach, dazu Plapperlaute (E-286)
            let key = (conv.dialog.clone(), conv.node.clone());
            if a.reveal_key != key {
                a.reveal_key = key;
                a.reveal = 0.0;
            }
            let text: Vec<char> = crate::adventure_hud::with_keys(
                node.text.get(code),
                &self.settings.bindings,
                &self.lang,
            )
            .chars()
            .collect();
            a.reveal_total = text.len();
            #[allow(clippy::cast_precision_loss)]
            let total = text.len() as f32;
            let before = a.reveal;
            a.reveal = (a.reveal + dt * REVEAL_PER_SECOND).min(total);
            let voice = content
                .characters
                .get(speaker)
                .map_or(ELORA_VOICE, |c| c.voice);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (from, to) = (before as usize, a.reveal as usize);
            if voice > 0.0 {
                for i in from..to {
                    if i % SYLLABLE == 0 && text.get(i).is_some_and(|c| c.is_alphanumeric()) {
                        #[allow(clippy::cast_precision_loss)]
                        let wobble = ((i * 7919) % 13) as f32 / 100.0 - 0.06;
                        self.ui_cues.push(
                            elora_audio::Cue::global(elora_audio::Sound::Voice)
                                .pitched(voice * (1.0 + wobble)),
                        );
                    }
                }
            }
            let name = content
                .characters
                .get(speaker)
                .map_or("Elora", |c| c.name.get(code));
            let choices = conv
                .choices(content, &session.save)
                .into_iter()
                .map(|i| (i, &node.choice[i]))
                .collect();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let view = crate::adventure_hud::DialogView {
                speaker,
                name,
                text: &node.text,
                choices,
                shown: a.reveal as usize,
            };
            chosen = crate::adventure_hud::dialog(
                &mut ui,
                &self.creature_art,
                &self.lang,
                code,
                &view,
                screen,
                &self.settings.bindings,
            );
        }
        // Abenteuer-Menü
        let mut menu_cmd = None;
        if let Some(st) = &mut a.menu {
            let tint = crate::skins::tint(
                self.net.skin,
                elora_sim::Team::None,
                false,
                crate::draw::team_color,
            );
            let data = crate::adventure_menu::MenuData {
                lang: &self.lang,
                code,
                content: &session.content,
                save: &session.save,
                art: &self.creature_art,
                figure: &self.figure_art,
                tint: &tint,
                screen,
            };
            menu_cmd = crate::adventure_menu::draw(&mut ui, &data, st);
        }
        // Kapitel geschafft: Gewinn-Bildschirm
        let mut victory_done = false;
        if let Some((id, at)) = &a.victory
            && let Some(area) = session.content.areas.iter().find(|x| &x.id == id)
        {
            let mut springs = [None; 5];
            for (slot, x) in springs
                .iter_mut()
                .zip(session.content.areas.iter().filter(|x| x.id != "tauwinkel"))
            {
                // die eben beruhigte Quelle singt schon
                if x.freed(&session.save) || &x.id == id {
                    *slot = Some(crate::adventure_menu::hex(&x.color));
                }
            }
            let text = |t: &Option<elora_adventure::data::Text>| {
                t.as_ref().map_or(String::new(), |t| t.get(code).to_owned())
            };
            let stats = self.lang.f(
                "victory.stats",
                &[
                    ("level", &session.save.level),
                    ("time", &play_time(&self.lang, session.save.play_time_secs)),
                ],
            );
            let view = crate::adventure_hud::VictoryView {
                chapter: area.chapter.unwrap_or(0),
                area: area.name.get(code),
                line: &text(&area.victory),
                honor: &text(&area.honor),
                color: crate::adventure_menu::hex(&area.color),
                springs,
                stats: &stats,
                time: at.elapsed().as_secs_f32(),
            };
            victory_done = crate::adventure_hud::victory(&mut ui, &self.lang, &view, screen);
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
        if let Some(c) = menu_cmd {
            self.adventure_command(c);
        }
        if victory_done {
            self.close_victory();
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

/// Text zur Zeit `secs` (Menü).
pub fn play_time(lang: &Lang, secs: u64) -> String {
    lang.f(
        "adventure.time",
        &[("h", &(secs / 3600)), ("m", &((secs / 60) % 60))],
    )
}
