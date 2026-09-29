# M2 – Kampf lokal: Umsetzungsplan

Status: **abgeschlossen** (E-056) · angenommen (E-050–E-055) · Grundlage: [`06-roadmap.md`](06-roadmap.md) M2, E-016, E-023, E-025

## Ziel

In der Sandbox lassen sich Hammer, Laser und Granate benutzen, gegen Trainings-Dummies und gegen sich selbst (Rocket-Jump). HP, Rüstung, Pickups, Tod und Respawn funktionieren. Die Abnahme ist bestanden, wenn du bestätigst, dass sich die Waffen treffsicher und wuchtig anfühlen.

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M2.1 ✅ | Waffen-Kern | `elora-sim` | Waffen-Tuning (T-18 bis T-27), Waffenzustand pro Figur (Besitz, Munition, Reload-Timer, aktive Waffe, Wechsel-Warteschlange), Feuerlogik (Klick bzw. Dauerfeuer), Eingabe erweitert um Feuer und Waffenwahl | Unit-Tests: Feuerraten, Munition, Wechsel erst nach Reload |
| M2.2 ✅ | Hammer | `elora-sim` | Treffer-Bereich, Sichtlinie, Knockback (T-19), Sperre nach Treffer | Tests: Knockback-Richtung, kein Treffer durch Wände |
| M2.3 ✅ | Granate & Explosion | `elora-sim` | Projektil mit analytischer Flugbahn (T-24), Kollision mit Wand und Spieler, Explosion mit Radius, Kraft und Schaden (T-25), Eigenschaden (T-26) → Rocket-Jump | Tests: Flugbahn, Explosions-Abfall, Rocket-Jump-Höhe |
| M2.4 ✅ | Laser | `elora-sim` | Hitscan mit Reichweite (T-21), Abprall mit Verzögerung (T-22), Treffer am ersten Spieler | Tests: Reichweite, Abprall, Energieverbrauch |
| M2.5 ✅ | Leben & Tod | `elora-sim` | HP und Rüstung (T-28), Schadensverteilung wie im Original, Tod durch Schaden oder Todes-Tile, Ereignisse (Treffer, Tod) | Tests: Rüstungslogik, Eigenschaden |
| M2.6 ✅ | Pickups & Respawn | `elora-sim` | Pickups aus der Karte (Herz, Rüstung, Laser, Granate) mit Respawn-Timer (T-29), Respawn-Verzögerung (T-30), Spawnpunkt-Wahl, Startausrüstung nur Hammer (E-025) | Tests: Aufnahme-Regeln, Timer |
| M2.7 ✅ | Trainings-Dummies | `elora-sim` + Client | Ziele zum Testen, keine Bots (E-035), siehe D-M2-04 | Manuell |
| M2.8 ✅ | Darstellung & HUD | Client | Platzhalter für Projektile, Laserstrahl, Explosion, Pickups, Waffe in der Hand; HUD mit HP, Rüstung, Munition und aktiver Waffe; Debug-Panel um die Waffenwerte erweitert | Sichtprüfung |
| M2.9 ✅ | Determinismus | `elora-sim` | Aufzeichnung und Golden-Tests um Feuer und Waffenwahl erweitert (Formatversion 2) | Tests grün |
| M2.10 ✅ | Abnahme | – | Du spielst die Sandbox | Deine Abnahme |

Nach jedem Schritt: `cargo xtask check` grün, dann ein Commit.

## Technische Festlegungen (Vorschlag)

- **Alles, was das Spielgeschehen beeinflusst, liegt in `elora-sim`:** Waffen, Projektile, Laser, Schaden, Pickups, Tod. Nur so kann der Client in M3 dieselbe Logik zur Vorhersage nutzen. Spielregeln wie Punkte, Teams und Runden folgen in M4 in `elora-game`.
- **Ereignisse** (Schuss, Treffer, Explosion, Tod, Pickup) liefert die Simulation als Liste pro Tick. Sie sind die Grundlage für Effekte und Sounds (M5) und später für Netzwerk-Events.
- **Werte:** Alle Waffenwerte kommen in `Tuning` (live im Panel einstellbar, gespeichert in `tuning.toml`). Festwerte aus dem Original, die nicht in `04-tuning.md` stehen, bleiben wie im Original (Hammer-Radius, Sperre nach Hammer-Treffer, Pickup-Radius, 125 ms Sperre ohne Munition). Sie werden als Konstanten dokumentiert.

## Entscheidungen zu M2

| # | Frage | Original-Verhalten (0.7) |
|---|---|---|
| D-M2-01 → E-051 | Waffenwahl: Tasten | Mausrad vor/zurück, Zahlentasten 1–5 (Hammer, Pistole, Shotgun, Granate, Laser) |
| D-M2-02 → E-051 | Reihenfolge und Nummern unserer 3 Waffen | – (Original: Hammer, …, Granate, Laser) |
| D-M2-03 → E-052 | Laser-Knockback | kein Knockback |
| D-M2-04 → E-053/E-054 | Art der Trainings-Dummies | – |
| D-M2-05 → E-055 | Selbstmord-Taste (`kill`) schon in M2? | vorhanden (Standard: K) |

## Hinweis zu T-30 (Respawn)

Im Original gilt: Respawn **frühestens 0,5 s** nach dem Tod, und zwar sobald die Feuertaste gedrückt ist – ohne Klick automatisch nach **3 s** (`CPlayer::Tick`). Umgesetzt wird genau dieses Verhalten; T-30 (0,5 s) ist die Mindestverzögerung. Dummies respawnen ohne Klick, also nach 3 s an ihrer Kartenposition.

## Umsetzungsnotizen

- **Steuerung:** Linke Maustaste schießt, 1/2/3 wählen Hammer/Granate/Laser (E-051), Mausrad blättert (hoch = vorige, runter = nächste Waffe, wie im Original). Tot: Feuertaste = Respawn (frühestens 0,5 s), sonst automatisch nach 3 s. R setzt Elora sofort an den besten Spawnpunkt.
- **Eingabe-Zähler:** Feuer und Mausrad sind wie im Original Zähler (`fire`, `next_weapon`, `prev_weapon`), damit sehr kurze Klicks nicht verloren gehen.
- **Tick-Reihenfolge** wie im Original: Eingaben/Klick-Schüsse → Projektile → Laser → Pickups → Figuren (Kräfte, dann Waffen) → Bewegung → Respawn. Der Reload-Timer wird im Schuss-Tick bereits einmal heruntergezählt (wie im Original).
- **Pickups** nimmt jede Figur auf, auch Dummies (wie im Original). Effektiver Aufnahme-Radius 40.
- **Dummies** zählen als Spieler-Slots (hookbar, treffbar), respawnen ohne Klick nach 3 s an ihrer Kartenposition.
- **Aufzeichnungen** haben jetzt Formatversion 2 (Feuer, Waffenwahl, Dummies, Pickups). Alte Aufzeichnungen (Version 1) werden beim Laden automatisch umgewandelt.
- **Golden-Tests:** `scripted` (Bewegung), `scripted-combat` (Waffen, Pickups, Rocket-Jumps, Tode) und deine Aufzeichnung aus der M1-Abnahme. Der Umbau der Welt wurde gegen die alte Golden-Datei geprüft: Die Bewegung ist bit-genau unverändert.

