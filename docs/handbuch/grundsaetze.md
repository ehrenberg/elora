# Grundsätze

Was aus Release 1 verbindlich bleibt – als Kurzfassung. Die Herleitung steht im
[Entscheidungslog von Release 1](../archiv/release-1/02-entscheidungen.md) (E-001 bis E-173);
neue Entscheidungen stehen im [Log von Release 2](../release-2/entscheidungen.md) (ab E-200).

## Zusammenarbeit

- **Der Projektinhaber trifft alle Entscheidungen.** Claude schlägt vor, begründet und fragt nach; keine Annahmen.
- Erkenntnisse und Entscheidungen werden in `docs/` festgehalten (E-002).
- Arbeit in **Meilensteinen** mit Plan → Freigabe → Umsetzung → **Abnahme im Playtest** (E-036). Es muss sich richtig anfühlen, nicht nur funktionieren.
- Nach jedem Schritt ist `cargo xtask check` grün, dann wird committet (Conventional Commits).
- Schlüssel und Geheimnisse kommen nie ins Repository (`server_key.toml`, API-Schlüssel nur als Umgebungsvariable, E-084).

## Spiel

| Bereich | Grundsatz | Herkunft |
|---|---|---|
| Ziel | Eigenes Spiel mit dem Spielgefühl von Teeworlds 0.7, eigene Identität „Elora“ | E-001, E-005, E-018 |
| Plattformen | Desktop: Linux, Windows, macOS | E-004 |
| Code | komplett neu in Rust, eigene Engine; Teeworlds dient nur als Referenz für Werte und Abläufe | E-007, E-009 |
| Simulation | deterministisch, 50 Ticks/s, `f32` mit Quantisierung je Tick | E-021 |
| Tuning | eigene, begründete Abweichungen vom Original; jeder Wert einzeln entschieden | E-015, E-022, [`tuning.md`](tuning.md) |
| Waffen | Hammer, Granatwerfer, Laser; Spawn nur mit Hammer | E-016, E-025 |
| Modi | DM, TDM, CTF, LMS, LTS, jeweils als Instagib | E-014, E-076 |
| Figur | Elora, Tropfenform „Wirbel“, Skins nur aus Farben (Augen, Körper, Füße) | E-085, E-094, E-095 |
| Bots | Bots liefern Eingaben wie Spieler (bisher nur Trainings-Dummies) | E-035, E-053 |

## Technik

| Bereich | Grundsatz | Herkunft |
|---|---|---|
| Grafik | wgpu + winit, eigener 2D-Vektor-Renderer (lyon), Assets als SVG | E-011, E-030, E-033 |
| Oberfläche | eigene Spiel-UI „hell & weich“; egui (dunkel) für Debug-Panel und Editor | E-031, E-150 |
| Sprache | Deutsch und Englisch, Texte in `assets/lang/` | E-114 |
| Ton | kira; Sounds aus CC0-Quellen (Kenney), neue Sounds liefert der Projektinhaber | E-032, E-108, E-109 |
| Netzwerk | eigenes UDP-Protokoll: Token, Noise-Verschlüsselung, Server-Schlüssel wie SSH, Snapshots mit Delta; IPv4 und IPv6 | E-012, E-061, E-062, E-063 |
| Server-Liste | HTTP/JSON-Master über HTTPS, `https://elora.bastianswelt.de`; dedizierte Server tragen sich ein, gehostete nur auf Wunsch | E-112, E-127, E-166, E-170 |
| Karten | binäres Format `.emap`, Editor im Spiel, automatischer Download, eigene Karten im Benutzerverzeichnis | E-129, E-136, E-146, E-152 |
| Kartenlook | Stil A „Weich & lebendig“; Stein nur für nicht hookbare Wände | E-139, E-148 |
| Lizenzen | Code GPL-3.0, eigene Assets CC-BY-SA 4.0, fremde Assets nur mit passender Lizenz | E-020, E-027 |
| Werkzeuge | Rust-Version fixiert, `cargo xtask check` (fmt, clippy, nextest, deny), GitHub Actions für Prüfung und Release-Pakete | E-038, E-039, E-163 |
| Vertrieb | GitHub Releases; Versionen nach SemVer, Release 1 = 0.9.0 Beta | E-161, E-165 |
