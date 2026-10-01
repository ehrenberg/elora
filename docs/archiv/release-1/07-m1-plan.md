# M1 – Physik-Sandbox: Umsetzungsplan

Status: **abgeschlossen** (E-049) · angenommen (E-043–E-046) · Grundlage: [`06-roadmap.md`](06-roadmap.md) M1, E-013

## Ziel

Elora läuft, springt, macht den Doppelsprung und hookt auf der Testkarte. Alle Tuning-Werte lassen sich live einstellen. Die Abnahme ist bestanden, wenn du bestätigst, dass sich die Bewegung richtig anfühlt.

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M1.1 ✅ | Simulationskern | `elora-sim` | `Vec2` mit Quantisierung (E-021), `Tuning` (E-023), Tile-Kollision (`MoveBox`, Raycast für den Hook), Bewegung, Sprung, Doppelsprung, Velocity Ramp, Hook-Zustandsautomat | Unit-Tests: Sprunghöhen aus `../../handbuch/tuning.md` nachrechnen, Hook-Reichweite, keine Tunnel durch Wände |
| M1.2 ✅ | Karten-Loader | `elora-map` | Parser für `.emap.toml` (E-024), Validierung mit Zeile und Spalte, Testkarte `maps/sandbox.emap.toml` | Tests: gültige und ungültige Karten |
| M1.3 ✅ | Fenster & Renderer | `elora-render` | winit-Fenster, wgpu, Formen per lyon (Kreis, Rechteck, Linie; E-033), Kamera mit Sichtbereich (D-02) | Sichtprüfung |
| M1.4 ✅ | Game-Loop & Eingabe | `elora-client` | Fester Tick mit 50 TPS und Akkumulator, Interpolation zwischen Ticks, Tastenbelegung (D-05), Mauszielen, Kamera (D-01) | Die Sandbox ist spielbar |
| M1.5 ✅ | Debug-Werkzeuge | `elora-client` | egui-Panel: alle Tuning-Werte als Regler, Anzeige von Geschwindigkeit, Bodenkontakt und Hook-Zustand, Hot-Reload der Karte, Taste für Reset/Respawn, Speichern des Tunings (D-03) | Manuell |
| M1.6 ✅ | Determinismus | `elora-sim` | Input-Aufzeichnung → Golden-Datei mit dem End-Zustand, Test im `cargo xtask check` | Test grün |
| M1.7 ✅ | Abnahme | – | Du spielst die Sandbox, eventuell mit Nach-Tuning. Die finalen Werte kommen in `../../handbuch/tuning.md`. | Deine Abnahme |

Nach jedem Schritt: `cargo xtask check` grün, dann ein Commit.

## Technische Festlegungen (Vorschlag)

- **Mathe:** eigener, kleiner `Vec2` in `elora-sim` statt glam. Wir haben damit volle Kontrolle über jede Float-Operation und die Rundung, das ist wichtig für den Determinismus (E-021). `elora-sim` bleibt ohne externe Abhängigkeiten.
- **Portierung:** Die Bewegung (`CCharacterCore::Tick`/`Move`) und `MoveBox` folgen dem Teeworlds-Code als Referenz (E-007), mit unseren Tuning-Werten.
- **Abhängigkeiten, nur stabile Versionen:** wgpu, winit (stabile 0.30-Reihe statt der 0.31-Beta), egui, egui-wgpu, egui-winit, lyon, toml und serde, notify für den Hot-Reload, tracing, anyhow/thiserror, pollster. Die genauen Versionen ergeben sich daraus, was untereinander kompatibel ist.
- **Platzhalter-Optik:** Elora ist ein Kreis mit Blickrichtung, Tiles sind farbige Rechtecke je nach Art, der Hook ist eine Linie, dazu ein Fadenkreuz.

## Entscheidungen zu M1

| # | Frage | Original-Verhalten (0.7) |
|---|---|---|
| D-01 → E-044 | Kamera | Statisch (Standard): Kamera exakt auf der Figur, Fadenkreuz max. 400 Einheiten entfernt. Optional dynamisch mit Totzone 300, Folgefaktor 60 %, max. Mausdistanz 1000 |
| D-02 → E-045 | Sichtbereich | Sichtfläche 1150 × 1000 = 1,15 Mio. Einheiten², höchstens 1500 × 1050 Einheiten. Das Seitenverhältnis bestimmt Breite und Höhe. |
| D-03 → E-046 | Tuning speichern | – (im Original eine Server-Einstellung) |
| D-05 → E-043 | Tastenbelegung | A/D laufen, Leertaste springen, LMB schießen, RMB hooken |

## Umsetzungsnotizen

- **Sandbox starten:** `cargo run --bin elora` (Standardkarte `maps/sandbox.emap.toml`) oder `cargo run --bin elora -- pfad/zur/karte.emap.toml`.
- **Steuerung:** A/D, Leertaste, RMB (Hook), R Respawn, F1 Panel, F5 Aufzeichnung, Esc Maus freigeben / beenden.
- **Tuning:** Regler im Panel; *Speichern* schreibt `tuning.toml` im Arbeitsverzeichnis (E-046).
- **Aufzeichnungen (M1.6):** F5 setzt Elora auf den Spawn und zeichnet alle Eingaben auf; erneut F5 (oder R, Kartenänderung, Tuning-Änderung) beendet und speichert nach `crates/elora-sim/tests/recordings/rec-<zeit>.erec.toml`. Die Datei enthält Raster, Spawn, Tuning und Eingaben. Golden-Datei erzeugen: `ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features`. Danach prüft `cargo xtask check` bei jeder Änderung, dass die Simulation bit-genau gleich bleibt.
- **Platzhalter:** Elora wird in Hitbox-Größe (28) gezeichnet; die spätere Darstellungsgröße ist Teil von M5.

