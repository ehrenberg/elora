# Architektur & Code-Struktur

Stand: Release 1 (0.9.0) · Grundsätze: [`grundsaetze.md`](grundsaetze.md)

## 1. Leitprinzipien

1. **Die Simulation ist reine Logik.** Physik, Hook, Waffen und Spielregeln kennen weder Fenster noch Grafik noch Netzwerk. Server, Client (Vorhersage), Sandbox und Tests nutzen denselben Code.
2. **Abhängigkeiten zeigen in eine Richtung:** Programme → Engine-Crates → Simulation. Die Simulation hängt von nichts ab, was mit Plattform oder Ein-/Ausgabe zu tun hat.
3. **Determinismus ist getestet.** Gleiche Eingaben ergeben bit-genau denselben Zustand; Golden-Tests spielen aufgezeichnete Eingaben ab (`crates/elora-sim/tests/recordings/`).
4. **Logik ohne Fenster ist testbar.** Online-Client, Editor-Werkzeuge, Karten und Netz laufen in Tests ohne Grafik (Speichernetz `MemNetwork`, virtuelle Zeit).

## 2. Workspace

```
crates/
  elora-sim        Simulation: Welt, Kollision (Tile-Arten), Figur, Hook, Waffen, Pickups, Dummies, Fähigkeiten, Gegner, Beute, Tuning, Aufzeichnungen
  elora-game       Spielregeln: Modi, Punkte, Runden, Teams, Aufwärmen, Sudden Death
  elora-map        Kartenmodell und Format .emap (binär, zlib), Aussehen (Materialien, Deko, Ebenen, Envelopes, SVGs)
  elora-protocol   Nachrichten, Snapshots mit Delta, Huffman, Server-Info, übersetzbare Meldungen
  elora-net        UDP: Token, Noise-Handshake, Zuverlässigkeit, Fragmente, Info-Abfrage, IPv4/IPv6, Netz-Simulator
  elora-render     wgpu-Renderer: Formen, Meshes, SVG-Assets, Text, Kamera
  elora-audio      Sounds: prozeduraler Generator, Zuordnung zu Ereignissen, Wiedergabe (kira)
  elora-adventure  Abenteuer: Inhalte als Daten, Spielstand, Stufen, Fähigkeitenbaum, Inventar, Läden, Ausbau, Speichern
apps/
  elora-client     das Spiel (lib: Online-Client, Szene, Kartenspeicher; bin: Fenster, Menüs, HUD, Editor)
  elora-server     dedizierter Server (lib: Spielserver, Konsole, Abstimmungen, Pfade; bin: Programm, Master-Anmeldung)
  elora-master     Master-Server für die Internet-Liste (HTTP/JSON, UDP-Prüfung)
xtask/             cargo xtask: check, package, map-dump, svg-preview, sound-preview/-import, train-huffman, net-stats
deploy/            Betrieb: master/ (systemd, Docker, Proxy), master-php/ (Webspace: Master + Projektseite)
tools/design/      Python-Generatoren für Entwürfe, Kartengrafik und Release-Karten
assets/            SVGs (Figur, Items, Emotes, Karten), Sounds, Schriften, Sprachen; adventure/ mit Inhalten des Abenteuers
maps/              mitgelieferte Karten (.emap)
```

### Client (`apps/elora-client/src`)

| Bereich | Module |
|---|---|
| Ablauf | `main.rs` (App, Bildschirme Menü/Spiel/Editor, Eingaben), `app_menu.rs`, `app_editor.rs` |
| Spiel lokal | `sandbox.rs` (Training, Hot-Reload, Aufzeichnung, Testspiel) |
| Spiel online | `online.rs` (lib: Snapshots, Vorhersage, Interpolation, Karten-Download), `connection.rs`, `map_store.rs` (lib) |
| Darstellung | `draw.rs`, `map_view.rs` (Ebenen, Parallax, Zwischenspeicher), `map_art.rs` (Auto-Kanten), `figure.rs`, `items.rs`, `effects.rs`, `emotes.rs`, `skins.rs` |
| Oberfläche | `ui.rs` (eigenes Toolkit), `menu*.rs`, `hud.rs`, `game_ui.rs`, `debug_ui.rs` + `gui.rs` (egui), `lang.rs` |
| Editor | `editor/` – Zustand und Verlauf (`mod.rs`), Werkzeuge (`tools.rs`), Aussehen (`look.rs`), Oberfläche (`panel*.rs`), Ansicht (`view.rs`) |
| Sonstiges | `browser.rs`, `hosting.rs`, `settings.rs`, `bindings.rs`, `controls.rs`, `sound.rs`, `tuning_file.rs` |

## 3. Laufzeit

- **Tick-Modell:** feste 50 Ticks/s. Der Client rendert mit beliebiger Bildrate und interpoliert zwischen zwei Zuständen (Akkumulator).
- **Online:** Server schickt Snapshots (25 Hz, LAN 50 Hz) als Delta; der Client sagt die eigene Figur samt Waffen voraus und interpoliert andere. Karten kommen beim Beitritt in Teilen, geprüft per BLAKE2s (`MapInfo` → `MapRequest`/`MapChunk` → `MapReady` → `Welcome`).
- **Dateien:** Mitgelieferte Daten über `elora_server::paths::resolve` (Arbeitsverzeichnis, `ELORA_DATA`, neben dem Programm, `../share/elora`, `../Resources`); Einstellungen, Tuning, Server-Schlüssel im Einstellungsordner; eigene und geladene Karten unter `~/.local/share/elora`.
- **Protokollversion:** 6 (bei Änderungen erhöhen; Master-PHP `config.php` anpassen).

## 4. Standards & Werkzeuge

| Bereich | Standard |
|---|---|
| Sprache | Rust 2024, Version in `rust-toolchain.toml` |
| Prüfung | `cargo xtask check`: rustfmt, clippy (pedantic, `-D warnings`), cargo-nextest, cargo-deny; dasselbe in GitHub Actions bei jedem Push |
| Abhängigkeiten | zentral in `[workspace.dependencies]`, Lizenzen und Advisories über cargo-deny |
| Fehler | `thiserror` in Bibliotheken, `anyhow` in Programmen; Logging mit `tracing` |
| Sichtprüfung | ignorierte Tests erzeugen SVGs unter `target/`, `cargo xtask svg-preview` rastert sie |
| Release | Tag `v<version>` → GitHub Actions baut Pakete und legt einen Release-Entwurf an; Notizen in `docs/releases/` |
| Commits | Conventional Commits |
