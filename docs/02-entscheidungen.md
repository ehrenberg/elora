# Entscheidungslog

Alle Entscheidungen trifft der Projektinhaber. Hier wird jede Entscheidung mit Datum und Begründung festgehalten. Offene Punkte bleiben offen, bis sie entschieden sind – es werden **keine Annahmen** getroffen.

## Getroffene Entscheidungen

| # | Datum | Thema | Entscheidung | Begründung |
|---|---|---|---|---|
| E-001 | 2026-09-25 | Ziel | Klon von Teeworlds mit identischem Spielgefühl | Vorgabe Projektinhaber |
| E-002 | 2026-09-25 | Dokumentation | Erkenntnisse im Ordner `docs/` | Vorgabe Projektinhaber |
| E-003 | 2026-09-25 | Ziel (O-01) | Veröffentlichung (öffentliches Release, eigene Identität) | Entscheidung Projektinhaber |
| E-004 | 2026-09-25 | Plattform (O-02) | Desktop: Linux, Windows, macOS | Entscheidung Projektinhaber |
| E-005 | 2026-09-25 | Referenzversion (O-03) | Teeworlds 0.7 | Entscheidung Projektinhaber |
| E-006 | 2026-09-25 | Assets (O-05) | Eigene Assets, eigener Stil | Entscheidung Projektinhaber |
| E-007 | 2026-09-25 | Code-Herkunft (O-23) | Komplett neu, Teeworlds-Quellcode dient als Referenz für Werte/Algorithmen | Entscheidung Projektinhaber |
| E-008 | 2026-09-25 | Protokoll (O-04) | Eigenes Netzwerkprotokoll, keine Kompatibilität zum Original | Entscheidung Projektinhaber |
| E-009 | 2026-09-25 | Sprache/Engine (O-07) | Rust mit eigener Engine (Server und Client teilen Code) | Entscheidung Projektinhaber |
| ~~E-010~~ | 2026-09-25 | Lizenz (O-22) | ~~Open Source, permissiv~~ → **ersetzt durch E-020** | Entscheidung Projektinhaber |
| E-011 | 2026-09-25 | Rendering (O-25) | wgpu + winit, eigener 2D-Renderer | Entscheidung Projektinhaber |
| E-012 | 2026-09-25 | Netzwerk (O-09) | Eigenes UDP-Protokoll (Snapshots, Delta-Kompression, eigene Zuverlässigkeitsschicht) | Entscheidung Projektinhaber |
| E-013 | 2026-09-25 | Erster Meilenstein (O-14/O-21) | Lokale Physik-Sandbox: ein Tee, Testkarte, Laufen/Springen/Hook – Feeling ohne Netzwerk abstimmen | Entscheidung Projektinhaber |
| E-014 | 2026-09-25 | Spielmodi Release 1 (O-12) | DM, TDM, CTF, LMS, LTS, Instagib | Entscheidung Projektinhaber |
| E-015 | 2026-09-25 | Physikwerte (O-30) | Original-Tuning **nicht** 1:1 übernehmen, sondern etwas abweichen (Umfang/Richtung → O-32) | Entscheidung Projektinhaber; eigene Identität |
| E-016 | 2026-09-25 | Waffen Release 1 (O-13) | Hammer, Laser, Granate | Entscheidung Projektinhaber |
| E-017 | 2026-09-25 | Testkarte (O-31) | Einfaches Textformat (Syntax → O-33) | Entscheidung Projektinhaber |
| E-018 | 2026-09-25 | Projektname (O-06) | **Elora** – zugleich Name der spielbaren Figur; Abgrenzung zu Teeworlds | Entscheidung Projektinhaber |
| E-019 | 2026-09-25 | Code-Struktur (O-29) | Aufteilung nach professionellem Rust-Standard gemäß [`03-architektur.md`](03-architektur.md) – **bestätigt** | Entscheidung Projektinhaber |
| E-020 | 2026-09-25 | Lizenz (O-24) | **GPL-3.0** (Copyleft), ersetzt E-010 | Entscheidung Projektinhaber |
| E-021 | 2026-09-25 | Physik-Arithmetik (O-28) | `f32` mit Quantisierung pro Tick (wie Original) | Entscheidung Projektinhaber |
| E-022 | 2026-09-25 | Vorgehen Physikwerte (O-32) | Claude schlägt pro Wert eine Abweichung mit Begründung vor, Projektinhaber entscheidet einzeln → [`04-tuning.md`](04-tuning.md) | Entscheidung Projektinhaber |

| E-023 | 2026-09-25 | Tuning (O-32) | Alle Vorschläge T-01 bis T-30 aus [`04-tuning.md`](04-tuning.md) angenommen | Entscheidung Projektinhaber |
| E-024 | 2026-09-25 | Karten-Textformat (O-33) | Vorschlag aus [`05-kartenformat.md`](05-kartenformat.md) angenommen (TOML + ASCII-Raster, Legende, Endung `.emap.toml`) – **nur für Test- und Entwicklungskarten** | Entscheidung Projektinhaber; für Release 1 zu einfach (keine Grafik-Layer) |
| E-025 | 2026-09-25 | Startausrüstung (O-35) | Elora spawnt **nur mit Hammer**; Laser und Granate ausschließlich per Pickup | Entscheidung Projektinhaber; Pickups und Kartenkontrolle werden wichtig |
| E-026 | 2026-09-25 | Instagib-Regeln | Klassisch: nur Laser, unendliche Munition, ein Treffer tötet, keine Pickups | Entscheidung Projektinhaber |
| E-027 | 2026-09-25 | Asset-Lizenz (O-36) | **CC-BY-SA 4.0** für eigene Grafiken/Sounds | Entscheidung Projektinhaber; Copyleft passend zu GPL-3.0 |
| E-028 | 2026-09-25 | Release-Kartenformat + Editor (O-10/O-16) | **Eigenes Format + eigener, ins Spiel integrierter Editor** (wie Teeworlds) | Entscheidung Projektinhaber; maximale Kontrolle |
| E-029 | 2026-09-25 | Figur & Skins (O-34/O-19) | Elora ist die Basisfigur; **Skin-System aus Teilen** (Körper, Augen, Deko u. ä., wie 0.7) inkl. Community-Skins | Entscheidung Projektinhaber |
| E-030 | 2026-09-25 | Grafikstil | **Flat/Vektor**: klare Formen, moderne Palette, auflösungsunabhängig | Entscheidung Projektinhaber; Abgrenzung zu Teeworlds |
| E-031 | 2026-09-25 | UI (O-27) | **egui** für Editor, Konsole, Debug-Regler; **eigene Spiel-UI** für Hauptmenü, Server-Browser, HUD | Entscheidung Projektinhaber |
| E-032 | 2026-09-25 | Audio (O-26) | **kira** | Entscheidung Projektinhaber |
| E-033 | 2026-09-25 | Vektor-Pipeline (O-38) | **Laufzeit-Tessellierung** (z. B. lyon → Dreiecke → wgpu), echt auflösungsunabhängig, dynamische Verformung möglich | Entscheidung Projektinhaber |
| E-034 | 2026-09-25 | Hosting (O-11) | Zunächst **nur lokales Git**, Hosting später | Entscheidung Projektinhaber |
| E-035 | 2026-09-25 | Bots (O-15) | **Nach Release 1**; Architektur sieht sie vor (Bots liefern Inputs wie Spieler) | Entscheidung Projektinhaber |
| E-036 | 2026-09-25 | Meilensteine (O-21) | Claude schlägt Roadmap bis Release 1 vor, Projektinhaber entscheidet je Meilenstein → [`06-roadmap.md`](06-roadmap.md) | Entscheidung Projektinhaber |
| E-037 | 2026-09-25 | Roadmap (O-21) | Meilensteine M0–M8 aus [`06-roadmap.md`](06-roadmap.md) angenommen | Entscheidung Projektinhaber |
| E-038 | 2026-09-25 | Rust-Toolchain | Umstieg auf **rustup**; Version fixiert in `rust-toolchain.toml` | Entscheidung Projektinhaber; reproduzierbar für alle Entwickler |
| E-039 | 2026-09-25 | Lokale Prüfungen (O-41) | **`cargo xtask`** (Rust-Programm im Workspace, plattformunabhängig) | Entscheidung Projektinhaber |
| E-040 | 2026-09-25 | Zusatz-Tools | **cargo-deny** (Lizenzen, Advisories) und **cargo-nextest** (Tests) | Entscheidung Projektinhaber |
| E-041 | 2026-09-25 | Workspace-Ort | Direkt im Projektordner (`Cargo.toml`, `crates/`, `apps/`, `docs/` im Root) | Entscheidung Projektinhaber |
| E-042 | 2026-09-25 | M0 Abnahme | M0 Projekt-Setup abgenommen (Commit `5ab9055`) | Entscheidung Projektinhaber |
| E-043 | 2026-09-25 | M1-Plan | Plan aus [`07-m1-plan.md`](07-m1-plan.md) inkl. technischer Festlegungen angenommen | Entscheidung Projektinhaber |
| E-044 | 2026-09-25 | Kamera (D-01) | **Statisch** wie 0.7-Standard: Kamera exakt auf Elora (korrigiert, siehe Analyse §9) | Entscheidung Projektinhaber |
| E-045 | 2026-09-25 | Sichtbereich (D-02) | Start mit Original (1,15 Mio. Einheiten², max. 1500 × 1050), als **Tuning-Regler** – finaler Wert in M1-Abnahme | Entscheidung Projektinhaber |
| E-046 | 2026-09-25 | Tuning speichern (D-03) | Sandbox speichert Werte in **`tuning.toml`**, wird beim Start geladen; Defaults bleiben im Code | Entscheidung Projektinhaber |
| E-047 | 2026-09-25 | Schriften | egui-Standardschriften (`epaint_default_fonts`, OFL-1.1 + Ubuntu Font Licence) **nicht** verwenden; stattdessen **Inter** (UI) und **JetBrains Mono** (Monospace), beide OFL-1.1, als Assets in `assets/fonts/` | Entscheidung Projektinhaber; keine Sonderlizenzen in Crate-Abhängigkeiten |
| E-048 | 2026-09-25 | Advisories | „unmaintained“-Meldungen von cargo-deny nur als **Warnung** (`-W unmaintained`), Sicherheitslücken bleiben Fehler. Anlass: `ttf-parser` (RUSTSEC-2026-0192, indirekt über egui) | Entscheidung Projektinhaber |

> **Hinweis zu E-020:** GPL-3.0 ist kompatibel mit der Teeworlds-Lizenz (zlib-artig) und mit MIT/Apache-lizenzierten Rust-Crates (wgpu, winit, …). Nicht kompatibel wären Abhängigkeiten unter GPL-2.0-only – `cargo-deny` prüft das. Die eigenen Assets (E-006) brauchen eine eigene Lizenz (→ O-36).

> **Hinweis zu E-007:** Die Teeworlds-Lizenz (zlib-artig) erlaubt Übernahme/Anpassung, verlangt aber, dass veränderte Versionen als solche gekennzeichnet sind und der Lizenzhinweis erhalten bleibt. Werden Algorithmen 1:1 nach Rust portiert, sollte der Teeworlds-Lizenzhinweis vorsorglich im Projekt mitgeführt werden (Datei `THIRD_PARTY_LICENSES`). Reine Zahlenwerte (Tuning) sind unkritisch.

## Offene Entscheidungen

### Grundlagen
- [x] ~~O-01 Ziel/Umfang~~ → E-003
- [x] ~~O-02 Plattform~~ → E-004
- [x] ~~O-03 Referenzversion~~ → E-005
- [x] ~~O-04 Kompatibilität~~ → E-008
- [x] ~~O-05 Assets~~ → E-006
- [x] ~~O-06 Projektname~~ → E-018
- [x] ~~O-22 Lizenz~~ → E-010
- [x] ~~O-23 Code-Herkunft~~ → E-007
- [x] ~~O-24 Konkrete Lizenz~~ → E-020
- [x] ~~O-36 Lizenz der Assets~~ → E-027

### Technik
- [x] ~~O-07 Programmiersprache / Engine~~ → E-009
- [x] ~~O-25 Grafik-/Fenster-Bibliothek~~ → E-011
- [x] ~~O-26 Audio-Bibliothek~~ → E-032
- [x] ~~O-27 UI-Lösung~~ → E-031
- [x] ~~O-38 Vektor-Pipeline~~ → E-033
- [ ] **O-40 Vektor-Quellformat** (SVG, eigenes Format, …) und Werkzeug zum Erstellen
- [x] ~~O-09 Netzwerk-Transport~~ → E-012
- [x] ~~O-28 Physik-Arithmetik~~ → E-021
- [x] ~~O-29 Rust-Workspace-Struktur~~ → E-019
- [x] ~~O-10 Release-Kartenformat~~ → E-028
- [ ] **O-37 Details Release-Kartenformat** (Layer-Modell, Binär/Text, Kompression, Asset-Einbettung) – später, vor den ersten echten Karten
- [x] ~~O-11 Versionskontrolle/Hosting~~ → E-034 (Repo-Struktur → E-019)
- [x] ~~O-41 CI ohne Hosting~~ → E-039

### Gameplay-Umfang
- [x] ~~O-12 Spielmodi~~ → E-014
- [x] ~~O-13 Waffen~~ → E-016
- [x] ~~O-14 Multiplayer vs. Sandbox zuerst~~ → E-013
- [x] ~~O-30 Physikwerte~~ → E-015
- [x] ~~O-31 Sandbox-Testkarte~~ → E-017
- [x] ~~O-32 Physikwerte im Einzelnen~~ → E-023
- [x] ~~O-33 Syntax des Karten-Textformats~~ → E-024
- [x] ~~O-34 Elora als Figur~~ → E-029
- [ ] **O-39 Skin-Aufbau im Detail** (Teile, Einfärbung, Format, Verteilung von Community-Skins)
- [x] ~~O-35 Startausrüstung~~ → E-025
- [x] ~~O-15 Bots~~ → E-035
- [x] ~~O-16 Map-Editor~~ → E-028 (Zeitpunkt → Meilensteine O-21)

### Features / Infrastruktur
- [ ] **O-17 Server-Browser / Master-Server**
- [ ] **O-18 Demos / Replays**
- [x] ~~O-19 Skins-System~~ → E-029
- [ ] **O-20 Konsole / Remote-Console**
- [ ] **O-42 Netzwerk-Zielwerte** (max. Spieler, Snapshot-Rate, Bandbreite)
- [ ] **O-43 Release-Karten** (Anzahl, Modi)
- [ ] **O-44 Vertrieb** (itch.io, Steam, Flathub, Website …)
- [x] ~~O-21 Meilensteine~~ → E-037
