//! Application state: construction of the `App`, player settings and tuning file.

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

use elora_render::ShapeBatch;

use crate::connection::{KNOWN_SERVERS_FILE, KnownServers};
use crate::controls::Controls;
use crate::debug_ui::NetUi;
use crate::sandbox::Sandbox;
use crate::settings::{self, Settings};
use crate::tuning_file::{TUNING_FILE, TuningFile};
use crate::{
    App, Screen, app_menu, creatures, editor, effects, emotes, figure, game_ui, hud, items, lang,
    map_view, menu, sound, weather,
};

/// Tuning and server key files in the settings folder (M8.2).
pub(crate) fn tuning_path() -> PathBuf {
    settings::config_file(TUNING_FILE)
}

pub(crate) fn known_servers_path() -> PathBuf {
    settings::config_file(KNOWN_SERVERS_FILE)
}

impl App {
    pub(crate) fn new(sandbox: Sandbox, file: &TuningFile, settings: Settings) -> Self {
        let mut controls = Controls::default();
        controls.sensitivity = settings.input.mouse_sensitivity;
        controls.auto_switch = settings.input.auto_switch;
        let mut net = NetUi::default();
        net.name.clone_from(&settings.player.name);
        net.skin = settings.player.skin();
        if let Some(r) = &sandbox.rules {
            net.sandbox_mode = Some(r.cfg.mode);
            net.sandbox_instagib = r.cfg.instagib;
        }
        Self {
            gfx: None,
            sandbox,
            online: None,
            known: KnownServers::load(&known_servers_path()),
            net,
            controls,
            view: file.view.into(),
            batch: ShapeBatch::default(),
            hud_batch: ShapeBatch::default(),
            hud: hud::Hud::new(),
            emotes: emotes::Emotes::new(),
            sounds: sound::Sounds::new(settings.audio),
            chat_heard: None,
            ui_cues: Vec::new(),
            owned_weapons: [false; 3],
            was_dazed: false,
            saturation: 1.0,
            weather: weather::WeatherView::default(),
            effects: effects::Effects::with_settings(settings.effects),
            figures: figure::Figures::default(),
            figure_art: figure::FigureArt::load(),
            item_art: items::ItemArt::load(),
            creature_art: creatures::CreatureArt::load(),
            map_view: map_view::MapView::default(),
            no_skins: BTreeMap::new(),
            last_frame: Instant::now(),
            fps: 0.0,
            frame_times: VecDeque::new(),
            cursor_grabbed: false,
            show_panel: false,
            status: String::new(),
            error: None,
            chat_input: game_ui::ChatInput::default(),
            scoreboard: false,
            killfeed: VecDeque::new(),
            lang: lang::Lang::new(settings.language),
            settings,
            screen: Screen::Menu,
            editor: None,
            editor_test: None,
            editor_area: editor::panel::AreaInfo::default(),
            menu: menu::Menu::default(),
            maps: app_menu::map_names(),
            bind_capture: None,
            adventure: None,
            intro: None,
        }
    }

    /// Take the current values from game and panel into the settings and save them.
    pub(crate) fn save_settings(&mut self) {
        let s = &mut self.settings;
        s.player.name.clone_from(&self.net.name);
        s.player.set_skin(self.net.skin);
        s.input.mouse_sensitivity = self.controls.sensitivity;
        s.input.auto_switch = self.controls.auto_switch;
        s.effects = self.effects.settings;
        s.audio = self.sounds.settings;
        let path = settings::settings_path();
        match s.save(&path) {
            Ok(()) => tracing::info!("settings saved: {}", path.display()),
            Err(e) => tracing::warn!("{e:#}"),
        }
    }

    pub(crate) fn tuning_file(&self) -> TuningFile {
        TuningFile {
            physics: self.sandbox.world.tuning.clone(),
            view: self.view.into(),
        }
    }
}
