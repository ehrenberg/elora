# Roadmap bis Release 1

Status: **angenommen** (E-037, 2026-09-25)

## Prinzipien

- **Zuerst das Gefühl, dann alles andere.** Jeder Meilenstein endet mit einer **Abnahme durch den Projektinhaber**: Es muss sich richtig anfühlen, nicht nur funktionieren.
- **Immer spielbar.** Nach jedem Meilenstein gibt es ein lauffähiges Programm.
- **Das Netzwerk kommt früh.** Prediction und Snapshots verändern die Architektur tief. Wir bauen es deshalb, bevor viel Gameplay darauf aufbaut (M3 vor den Spielmodi).
- **Kunst und Assets kommen spät.** Bis M4 reichen einfache Vektorformen, so können Gameplay-Änderungen keine fertige Grafik entwerten.

## Übersicht

```
M0 Setup ─► M1 Physik-Sandbox ─► M2 Kampf lokal ─► M3 Netzwerk ─► M4 Spielmodi
                                                                     │
      M8 Release ◄─ M7 Menüs & Infrastruktur ◄─ M6 Karten & Editor ◄─ M5 Look & Sound
```

| # | Meilenstein | Kern-Ergebnis | Offene Entscheidungen davor | Entscheidung |
|---|---|---|---|---|
| M0 | Projekt-Setup | Workspace kompiliert, lokale Prüfungen laufen | O-41 | ✅ |
| M1 | Physik-Sandbox | Elora läuft, springt und hookt, das Gefühl ist abgenommen | – | ✅ |
| M2 | Kampf lokal | Hammer, Laser, Granate, Schaden, Pickups, Tod und Respawn | – | ✅ |
| M3 | Netzwerk | Dedizierter Server, mehrere Clients, Prediction, im LAN spielbar | O-42 | ✅ |
| M4 | Spielmodi | DM, TDM, CTF, LMS, LTS, Instagib, Scoreboard, Chat | O-20 | ✅ |
| M5 | Look & Sound | Vektor-Renderer, Elora-Art, Skins, Partikel, Audio, HUD | O-39, O-40 | ✅ |
| M6 | Karten & Editor | Release-Kartenformat, integrierter Editor, erste echte Karten | O-37, O-43 | ✅ |
| M7 | Menüs & Infrastruktur | Hauptmenü, Einstellungen, Tastenbelegung, Server-Browser | O-17, O-18 | ✅ |
| M8 | Release | Pakete für 3 Betriebssysteme, Balancing, Playtests, Release 1 | O-44 | ✅ |

---

## M0 – Projekt-Setup

- Lokales Git-Repository (E-034), `.gitignore`, `LICENSE` (GPL-3.0), `THIRD_PARTY_LICENSES`
- Cargo-Workspace nach [`03-architektur.md`](03-architektur.md), zunächst nur mit den M1-Crates
- `rust-toolchain.toml`, rustfmt, clippy-Lints, cargo-deny
- Lokales Prüfskript (fmt + clippy + test + deny) als Ersatz für die CI (O-41)

**Abnahme:** `cargo build` und `cargo xtask check` laufen fehlerfrei.

**Stand 2026-09-25:** umgesetzt, `cargo xtask check` grün (fmt, clippy, nextest, deny). **Abgenommen** (E-042).

## M1 – Physik-Sandbox (E-013)

- `elora-sim`: Tuning (E-023), Quantisierung (E-021), Tile-Kollision, Bewegung, Sprung und Doppelsprung, Hook samt Zustandsautomat, Velocity Ramp
- `elora-map`: Textformat (E-024), Validierung, Hot-Reload
- `elora-render`: Fenster (winit), wgpu, Kamera mit Maus-Versatz, einfache Formen (Kreis für Elora, Rechtecke für Tiles, Linie für den Hook)
- Fester Tick-Loop mit 50 TPS und Render-Interpolation
- egui-Debug-Panel: alle Tuning-Werte live einstellbar, Anzeige von Geschwindigkeit und Zustand
- Golden-Tests für Determinismus (Input-Aufzeichnung → erwarteter Zustand)
- Testkarte `maps/sandbox.emap.toml`

**Abnahme:** Der Projektinhaber spielt die Sandbox und bestätigt das Bewegungsgefühl, eventuell nach Nach-Tuning.

## M2 – Kampf lokal

- Zielen per Maus, Waffenwechsel
- Hammer (Knockback T-19), Laser (Hitscan mit Abprall), Granate (ballistisch, Explosion, Rocket-Jump)
- HP und Rüstung, Eigenschaden, Tod, Respawn, Todes-Tiles
- Pickups mit Respawn-Timer, Startausrüstung nur Hammer (E-025)
- **Trainings-Dummies:** unbewegliche oder skriptgesteuerte Ziele zum Testen. Das sind keine Bots (E-035).
- Einfaches HUD per egui (Platzhalter)

**Abnahme:** Die Waffen fühlen sich treffsicher und wuchtig an, Rocket-Jumps und Hammer-Sprünge funktionieren.

## M3 – Netzwerk (E-008, E-012)

- `elora-net`: UDP, Verbindungsaufbau, Zuverlässigkeitsschicht, Timeouts
- `elora-protocol`: Input-Nachrichten, Snapshots, Delta-Kompression
- `elora-server`: dedizierter Server ohne Grafik
- Client: Prediction der eigenen Figur, Interpolation der anderen, Korrektur bei Abweichungen
- **Netzwerk-Simulator:** künstlicher Ping, Jitter und Paketverlust zum Testen
- Lokal Hosten und Direkt-Verbinden über IP (noch ohne Server-Browser)

**Abnahme:** 2 bis 8 Spieler im LAN. Bei simulierten 100 ms Ping fühlt sich die eigene Bewegung genauso an wie offline.

## M4 – Spielmodi (E-014)

- `elora-game`: Framework für Spielregeln, dazu DM, TDM, CTF, LMS, LTS und Instagib (E-026)
- Teams, Spawn-Logik, Flaggen, Runden, Warmup, Score- und Zeitlimit
- Scoreboard, Killfeed, Chat und Team-Chat
- Server-Konfiguration (Datei und Kommandozeile), Konsole (O-20)

**Abnahme:** Jeder Modus ist in einer Playtest-Runde spielbar.

## M5 – Look & Sound (E-029, E-030, E-032, E-033)

- Vektor-Renderer: lyon-Tessellierung, Transformationen, Verformung (Squash und Stretch)
- Elora-Design, Animationen (Laufen, Springen, Hook, Augen und Emotes)
- Skin-System aus Teilen (E-029, O-39)
- Partikel, Explosionen, Treffer-Feedback
- kira-Audio: alle Spielsounds, Lautstärke nach Entfernung
- Finales HUD (eigene Spiel-UI, E-031)

**Abnahme:** Stil und Feedback sind abgenommen. Elora ist klar von einem Tee zu unterscheiden.

## M6 – Karten & Editor (E-028)

- Release-Kartenformat (O-37): Game-Layer, Grafik-Layer, Parallax, Quads, Animationen
- Integrierter Editor (egui, E-031): Layer, Tiles, Entities, Testspielen direkt aus dem Editor
- Import von Textkarten (E-024) ins Release-Format
- Erste Release-Karten (O-43: wie viele und welche Modi)

**Abnahme:** Mit dem Editor lässt sich eine Karte von Grund auf bauen und spielen.

## M7 – Menüs & Infrastruktur

- Hauptmenü, Einstellungen (Grafik, Audio, Steuerung, Spieler und Skin), Tastenbelegung
- Server-Browser: LAN plus Internet über einen Master-Server (O-17)
- Eventuell Demos und Replays (O-18)

**Abnahme:** Ein neuer Spieler findet ohne Hilfe einen Server und kann spielen.

## M8 – Release 1 (E-003)

- Pakete und Installer für Linux, Windows und macOS (O-44: Vertriebskanäle)
- Balancing-Runden, öffentliche Playtests, Fehlerbehebung
- Lizenz- und Credits-Seite (GPL-3.0, CC-BY-SA 4.0, Drittanbieter)

**Abnahme:** Release 1 ist veröffentlicht.

---

## Neue offene Punkte aus dieser Roadmap

- **O-42 Netzwerk-Zielwerte:** maximale Spielerzahl pro Server, Snapshot-Rate, Ziel-Bandbreite
- **O-43 Release-Karten:** Anzahl und Modi für Release 1
- **O-44 Vertrieb:** Wo wird Release 1 veröffentlicht (itch.io, Steam, Flathub, eigene Website …)?
