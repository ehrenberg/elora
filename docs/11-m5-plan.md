# M5 – Look & Sound: Umsetzungsplan

Status: **angenommen** (E-080–E-097), in Umsetzung · Grundlage: [`06-roadmap.md`](06-roadmap.md) M5, E-006, E-027, E-029, E-030, E-031, E-032, E-033

## Ziel

Elora bekommt ihr eigenes Aussehen im Flat-/Vektorstil (E-030), mit Animationen, Skins, Effekten, Sounds und einem finalen HUD. Abnahme: Stil und Feedback stimmen, und Elora ist klar von einem Tee zu unterscheiden.

## Was das Original ausmacht (Quellcode `render.cpp`, `players.cpp`, `content.py`)

| Bereich | Original (0.7) |
|---|---|
| **Größe** | Figur mit **64 Einheiten** gezeichnet (Sprite), sichtbarer Körper ≈ 40–45, Hitbox nur 28 – die Figur wirkt größer, als sie trifft |
| **Aufbau** | Körper, Markierung, Dekoration, Hände, Füße, Augen; Umriss- und Füll-Durchgang; Schatten und oberer Umriss |
| **Animationen** | `idle`, `inair`, `walk` (Füße), `hammer_swing`; Körper rotiert leicht; Augen blicken in Zielrichtung |
| **Augen/Emotes** | normal, Schmerz, Freude, Überraschung, Wut, Blinzeln; 16 Emoticons über dem Kopf (Emote-Rad) |
| **Skins (0.7)** | Teile Körper, Markierung, Dekoration, Hände, Füße, Augen – je Teil eigene Farbe (HSL); Community-Skins als Teile-Sätze |
| **Effekte** | Rauchspur hinter Granaten, Explosion mit Rauch, Hammer-Treffer, Blut in Körperfarbe, Luftsprung-Wolken, Staub beim Landen |
| **Sounds** | 40 Sounds: je Waffe Schuss/Treffer, Laser-Abprall, Hook (Schleife, Wand, Spieler, kein Halt), Sprung, Luftsprung, Landung, Schmerz kurz/lang, Tod, Spawn, Pickups, keine Munition, Treffer-Bestätigung, Chat, CTF (fallen, zurück, Aufnahme eigene/fremde, Eroberung), Menü |
| **Positionaler Ton** | Lautstärke nach Entfernung zur Kamera, Stereo-Panorama |
| **HUD** | Herzen und Schilde oben links, Munition als Symbole, Waffen-Symbol; Punkte und Timer oben |

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M5.1 ✅ | Vektor-Renderer | `elora-render` | Transformationen (Verschieben, Drehen, Skalieren, Verformen), Verläufe, Antialiasing (MSAA), Formen-Cache (einmal tesselliert, oft gezeichnet), Kamera-Zoom | Sichtprüfung, Benchmark |
| M5.2 | Vektor-Assets | `elora-render` + `assets/` | Laden des Quellformats (D-M5-02), Farbschlüssel für die Einfärbung | Tests |
| M5.3 | Elora-Figur | Client | Figur aus Teilen (D-M5-03), Augen folgen dem Ziel, Füße laufen, Squash & Stretch (Sprung, Landung, Hook), Waffe in der Hand, Emotes | Sichtprüfung |
| M5.4 | Skins | Client + Protokoll | Skin-Teile und Farben (D-M5-04), Auswahl im Client, Übertragung an andere Spieler | Tests + Sichtprüfung |
| M5.5 | Welt-Optik | Client | Tiles mit Kanten und Ecken statt Rechtecken, Hintergrund, Pickups, Flaggen, Waffen als Grafik (D-M5-08) | Sichtprüfung |
| M5.6 | Effekte | Client | Partikel-System: Rauch, Explosionen, Treffer, Tod, Staub, Sprungwolken, Laserstrahl (D-M5-07) | Sichtprüfung |
| M5.7 | Audio | `elora-audio` (neu) | kira-Anbindung, Sounds aus Ereignissen, Lautstärke nach Entfernung + Stereo, Lautstärke-Regler (D-M5-05, D-M5-06) | Tests (Zuordnung Ereignis → Sound), Hörprobe |
| M5.8 | Finales HUD | Client | Eigene Spiel-UI (E-031) statt egui-Platzhalter: Leben, Rüstung, Munition, Waffen, Timer, Punkte, Killfeed, Chat, Scoreboard (D-M5-09) | Sichtprüfung |
| M5.9 | Emotes | Client + Protokoll | Emote-Rad und Emoticons über dem Kopf, übertragen an alle (D-M5-10) | Test + Sichtprüfung |
| M5.10 | Abnahme | – | Stil, Feedback, Klang | Deine Abnahme |

## Technische Festlegungen (Vorschlag)

- **Alles zur Laufzeit tesselliert (E-033), aber gecacht:** Eine Form wird einmal in Dreiecke zerlegt und dann per Transformation (Position, Drehung, Skalierung, Verformung) gezeichnet – so bleiben 64 Spieler mit Partikeln flüssig.
- **Einfärbung über Farbschlüssel:** Skin-Teile werden in Graustufen oder mit Platzhalterfarben gezeichnet; der Client ersetzt sie durch die gewählten Farben (wie HSL-Färbung im Original).
- **Neues Crate `elora-audio`** mit kira (MIT/Apache-2.0, E-032). Sounds werden nur aus den Ereignissen der Simulation und der Regeln ausgelöst – dieselben Ereignisse, die schon Effekte und Killfeed speisen.
- **Kein Einfluss auf die Simulation:** Größe der Darstellung, Animationen und Effekte sind reine Darstellung; Hitbox (28) und Physik bleiben unverändert.

## Entscheidungen zu M5

| # | Frage | Entscheidung |
|---|---|---|
| D-M5-01 | Wer erstellt Grafik? | E-080: Claude erzeugt SVG + Vektor-Bild-API für aufwendige Motive, Freigabe per Screenshot |
| – | Bild-API | E-084: Recraft (SVG), Schlüssel nur als Umgebungsvariable |
| D-M5-02 | Quellformat | SVG |
| D-M5-03 | Elora-Form | E-085: Tropfenform mit starkem Squash & Stretch |
| – | Darstellungsgröße | E-087: sichtbarer Körper ≈ 36 (Hitbox 28) |
| D-M5-04 | Skins | E-086: nur Farben + wenige Teile, keine Community-Skins; E-095: Augen, Körper, Füße, nur Farben; E-096: feste Palette; E-097: Bauchfleck aus Körperfarbe |
| – | Elora-Entwurf | E-094: Entwurf B „Wirbel“ |
| D-M5-05 | Sounds | E-081: prozedural generiert + CC0 gemischt |
| D-M5-06 | Musik | E-082: nur Menü (M7) |
| D-M5-07 | Zusatz-Effekte | E-088: Kamera-Wackeln, Treffer-Marker (abschaltbar) |
| D-M5-08 | Tiles/Hintergrund | E-089: schlicht |
| D-M5-09 | HUD | E-090: modern, am Fadenkreuz |
| D-M5-10 | Emotes | E-091: Emote-Rad mit 8 eigenen Emoticons |
| D-M5-11 | Reihenfolge | E-083: erst Look, dann Sound |

## Angepasste Schritte

- **M5.2 Vektor-Assets:** SVG laden (`usvg`, Apache-2.0/MIT) → lyon-Tessellierung; Farbschlüssel für die Einfärbung. Werkzeug `cargo xtask svg-preview` rastert SVGs zu PNG (für meine Selbstprüfung und deine Freigabe). Recraft-Anbindung als `xtask`-Befehl, nur mit gesetztem `RECRAFT_API_KEY`; erzeugte SVGs werden bereinigt und mit Quelle in `assets/SOURCES.md` vermerkt.
- **M5.3 Elora:** zuerst **3 Entwürfe der Tropfenform** als Bild zur Auswahl.
- **M5.4 Skins:** je Teil (Augen, Körper, Füße) eine Farbe aus fester Palette (E-095/E-096); übertragen werden nur drei Palettennummern. Palette als Entwurf zur Freigabe.
- **M5.5 Welt:** schlicht (E-089).
- **M5.7 Audio:** prozeduraler Generator (sfxr-artig, Parameter als Dateien in `assets/sounds/`) für UI/einfache Effekte; CC0-Sounds mit Quellenliste in `assets/SOURCES.md`.
- **M5.8 HUD:** 2–3 Entwürfe am Fadenkreuz zur Auswahl.
- **M5.9 Emotes:** 8 eigene Emoticons, Emote-Rad mit Taste E.

## Umsetzungsstand

- **M5.1 Vektor-Renderer:** `Mesh` (einmal tesselliert, lokale Koordinaten) + `Affine` (Verschieben, Drehen, Skalieren/Verformen) + `Paint` (Farbe, linearer Verlauf, Farbschlüssel mit Aufhellung) + `Tint` (Skin-Farben, Deckkraft). 4× MSAA mit Rückfall auf 1, wenn das Format es nicht kann. `Camera::zoomed`. Benchmark `cargo run --release -p elora-render --example bench_meshes`: 64 Figuren (je 316 Dreiecke) + 2000 Partikel = 34 224 Dreiecke, CPU-Aufbau ≈ 0,09 ms/Frame.
- **M5.3 Elora:** Entwurf B „Wirbel“ gewählt (E-094); Entwurfsblatt `docs/design/elora-entwuerfe.png`, Generator `tools/design/elora_entwuerfe.py`.

## Voraussetzungen vom Projektinhaber

- Für Recraft: ein API-Schlüssel (als Umgebungsvariable `RECRAFT_API_KEY`), erst wenn ein Motiv die API braucht.

