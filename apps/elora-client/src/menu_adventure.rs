//! Hauptmenü-Seite „Abenteuer“ (A1.6, E-219): drei Spielstand-Plätze.

// Layout-Code: `s` (Skalierung), `w`/`h`/`x`/`y` wie in menu.rs
#![allow(clippy::many_single_char_names)]

use elora_render::Align;
use elora_sim::Vec2;

use crate::app_adventure::{SlotView, play_time};
use crate::menu::{MenuAction, MenuCtx};
use crate::ui::{GRAY, GREEN, LOGO, Rect, TEXT, TEXT_DIM, Ui};

/// Plätze nebeneinander; Löschen mit Rückfrage.
#[allow(clippy::too_many_lines)] // Layout der drei Plätze
pub fn page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    area: Rect,
    confirm: &mut Option<usize>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let mut action = None;
    ui.label(
        lang.t("adventure.title"),
        Vec2::new(area.min.x + 12.0 * s, area.min.y + 30.0 * s),
        26.0,
        TEXT,
        Align::Left,
    );
    ui.label(
        lang.t("adventure.subtitle"),
        Vec2::new(area.min.x + 12.0 * s, area.min.y + 56.0 * s),
        12.0,
        TEXT_DIM,
        Align::Left,
    );
    let (w, h, gap) = (250.0 * s, 230.0 * s, 20.0 * s);
    for (i, slot) in cx.slots.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let card = Rect::new(
            area.min.x + 12.0 * s + i as f32 * (w + gap),
            area.min.y + 84.0 * s,
            w,
            h,
        );
        ui.card(card);
        let x = card.min.x + 18.0 * s;
        ui.label(
            &lang.f("adventure.slot", &[("n", &(i + 1))]),
            Vec2::new(x, card.min.y + 26.0 * s),
            15.0,
            LOGO,
            Align::Left,
        );
        let button =
            |k: f32| Rect::new(x, card.max.y - (k * 40.0 + 6.0) * s, w - 36.0 * s, 32.0 * s);
        if *confirm == Some(i) {
            ui.label(
                lang.t("adventure.confirm_delete"),
                Vec2::new(x, card.min.y + 70.0 * s),
                13.0,
                TEXT,
                Align::Left,
            );
            if ui.button(
                &format!("adv_yes{i}"),
                button(2.0),
                lang.t("adventure.yes"),
                crate::ui::ORANGE,
            ) {
                action = Some(MenuAction::AdventureDelete(i));
                *confirm = None;
            }
            if ui.button(
                &format!("adv_no{i}"),
                button(1.0),
                lang.t("adventure.no"),
                GRAY,
            ) {
                *confirm = None;
            }
            continue;
        }
        match slot {
            SlotView::Empty => {
                ui.label(
                    lang.t("adventure.empty"),
                    Vec2::new(x, card.min.y + 60.0 * s),
                    13.0,
                    TEXT_DIM,
                    Align::Left,
                );
                if ui.button(
                    &format!("adv_new{i}"),
                    button(1.0),
                    lang.t("adventure.new"),
                    GREEN,
                ) {
                    action = Some(MenuAction::AdventureNew(i));
                }
            }
            SlotView::Saved {
                level,
                map,
                play_secs,
                glanz,
            } => {
                let lines = [
                    (lang.f("adventure.level", &[("n", level)]), 14.0, TEXT),
                    (map.clone(), 12.0, TEXT_DIM),
                    (play_time(lang, *play_secs), 12.0, TEXT_DIM),
                    (lang.f("adventure.glanz", &[("n", glanz)]), 12.0, TEXT_DIM),
                ];
                for (k, (t, size, color)) in lines.iter().enumerate() {
                    #[allow(clippy::cast_precision_loss)]
                    let y = card.min.y + (60.0 + k as f32 * 22.0) * s;
                    ui.label(t, Vec2::new(x, y), *size, *color, Align::Left);
                }
                if ui.button(
                    &format!("adv_go{i}"),
                    button(2.0),
                    lang.t("adventure.continue"),
                    GREEN,
                ) {
                    action = Some(MenuAction::AdventureContinue(i));
                }
                if ui.button(
                    &format!("adv_del{i}"),
                    button(1.0),
                    lang.t("adventure.delete"),
                    GRAY,
                ) {
                    *confirm = Some(i);
                }
            }
            SlotView::Damaged(e) => {
                ui.label(
                    lang.t("adventure.damaged"),
                    Vec2::new(x, card.min.y + 60.0 * s),
                    13.0,
                    crate::ui::ORANGE,
                    Align::Left,
                );
                ui.label(
                    e,
                    Vec2::new(x, card.min.y + 82.0 * s),
                    10.0,
                    TEXT_DIM,
                    Align::Left,
                );
                if ui.button(
                    &format!("adv_del{i}"),
                    button(1.0),
                    lang.t("adventure.delete"),
                    GRAY,
                ) {
                    *confirm = Some(i);
                }
            }
        }
    }
    action
}
