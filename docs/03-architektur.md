# Architektur & Code-Struktur

Status: **bestätigt** (E-019 in [`02-entscheidungen.md`](02-entscheidungen.md)) · Lizenz: GPL-3.0 (E-020) · Arithmetik: `f32` + Quantisierung (E-021)

Grundlage: E-009 (Rust, eigene Engine), E-011 (wgpu + winit), E-012 (eigenes UDP-Protokoll), E-013 (zuerst Sandbox).

## 1. Leitprinzipien

1. **Die Simulation ist reine Logik.** Physik, Hook, Waffen und Spielregeln kennen weder Fenster noch Grafik noch Netzwerk. Server, Client (für die Vorhersage) und Tests verwenden exakt denselben Code.
2. **Abhängigkeiten zeigen nur in eine Richtung:** Apps → Engine-Crates → Simulation → Grundlagen. Die Simulation hängt von nichts ab, was mit Plattform oder Ein-/Ausgabe zu tun hat.
3. **Determinismus ist testbar.** Gleiche Inputs ergeben bit-genau denselben Zustand. Das sichern Golden-Tests (aufgezeichnete Input-Folgen gegen einen erwarteten End-Zustand) ab.
4. **Klein anfangen, bei Bedarf aufteilen.** Ein Crate entsteht erst, wenn es gebraucht wird. Für die Sandbox (E-013) reichen die Crates, die unten mit **M1** markiert sind.

## 2. Cargo-Workspace

```
elora/
├── Cargo.toml              # [workspace], gemeinsame Abhängigkeiten, Lints
├── rust-toolchain.toml     # fixierte Rust-Version 1.98.1 (E-038)
├── .cargo/config.toml      # Alias `cargo xtask`
├── rustfmt.toml / clippy.toml / deny.toml
├── crates/
│   ├── elora-sim/          # M1  deterministische Simulation: Welt, Kollision, Elora, Hook, Waffen, Tuning
│   ├── elora-map/          # M1  Karten-Datenmodell, Text-Format (E-024) und Release-Format (E-028)
│   ├── elora-render/       # M1  wgpu-2D-Renderer: Sprites, Tiles, Kamera, Partikel; egui-Integration (Debug/Editor, E-031)
│   ├── elora-input/        #     Eingabeabbildung (Tasten → Spieler-Input), konfigurierbar
│   ├── elora-audio/        #     Sound-Ausgabe (kira, E-032)
│   ├── elora-ui/           #     eigene Spiel-UI: Hauptmenü, Server-Browser, HUD (E-031)
│   ├── elora-protocol/     #     Netzwerknachrichten, Serialisierung, Snapshot/Delta
│   ├── elora-net/          #     UDP-Transport, Verbindungen, Zuverlässigkeitsschicht
│   ├── elora-game/         #     Spielmodi und Regeln (DM, TDM, CTF, LMS, LTS, Instagib), serverseitig
│   └── elora-editor/       #     Karten-Editor (E-028), wird vom Client eingebunden
├── apps/
│   ├── elora-client/       # M1  das Spiel (Fenster, Loop, Prediction, UI); in M1 = Sandbox
│   └── elora-server/       #     dedizierter Server ohne Grafik
├── assets/                 # eigene Grafiken, Sounds, Schriften (E-006)
├── maps/                   # Karten im Textformat (E-017)
├── xtask/                  # Entwicklungsaufgaben: `cargo xtask check` (E-039)
├── tools/                  # Hilfsprogramme (Asset-Pipeline, Map-Konverter, …)
├── docs/
├── LICENSE (GPL-3.0) / THIRD_PARTY_LICENSES
```

Aufbau der Simulation (`elora-sim`), Vorschlag:

```
src/
├── lib.rs
├── tuning.rs      # alle einstellbaren Werte an einer Stelle (E-015)
├── math.rs        # Vektoren (f32), Quantisierung pro Tick (E-021)
├── collision.rs   # Tile-Kollision, MoveBox, Raycast
├── character.rs   # Elora: Bewegung, Sprung, Doppelsprung
├── hook.rs        # Hook-Zustandsautomat
├── weapons/       # hammer.rs, laser.rs, grenade.rs (E-016)
├── world.rs       # GameWorld: Tick-Schleife, Entities
└── input.rs       # PlayerInput (pro Tick)
```

## 3. Standards & Werkzeuge

| Bereich | Standard |
|---|---|
| Edition | Rust 2024, Toolchain in `rust-toolchain.toml` fixiert |
| Formatierung | `rustfmt`, wird in der CI geprüft |
| Lints | `clippy` mit `-D warnings`, Lints zentral in `[workspace.lints]` |
| Abhängigkeiten | Versionen zentral in `[workspace.dependencies]`, `cargo-deny` prüft Lizenzen und Sicherheitsmeldungen |
| Tests | Unit-Tests je Crate, Determinismus-Golden-Tests in `elora-sim`, `cargo-nextest` |
| Benchmarks | `criterion` für die Simulation (die Tick-Dauer ist kritisch) |
| Fehlerbehandlung | `thiserror` in Bibliotheken, `anyhow` in Apps |
| Logging | `tracing` |
| CI | vorerst lokal über `cargo xtask check` (E-034, E-039); Build auf Linux/Windows/macOS, sobald Hosting entschieden ist |
| Versionierung | Git, Conventional Commits, SemVer |

## 4. Tick-Modell

- Feste Simulationsrate (Original: 50 TPS; unser Wert → Tuning, E-015).
- Der Client rendert mit beliebiger Framerate und interpoliert zwischen zwei Simulationszuständen (Fixed-Timestep-Loop mit Akkumulator).
