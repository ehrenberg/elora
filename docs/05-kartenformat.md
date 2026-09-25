# Kartenformat (Textformat)

Status: **angenommen** (E-024) · gilt **nur für Test- und Entwicklungskarten** · Grundlage: E-017

## 1. Ziele

1. **Von Hand schreibbar und lesbar.** Eine Karte lässt sich in jedem Texteditor bauen, man sieht die Karte direkt im Text.
2. **Diff-freundlich.** Änderungen erscheinen in Git zeilenweise.
3. **Robust parsbar.** Ein Standardformat statt eigener Syntax, klare Fehlermeldungen mit Zeile und Spalte.
4. **Erweiterbar.** Die Versionsnummer erlaubt später weitere Tile-Arten und Entities, ohne alte Karten zu brechen.
5. **Nur Gameplay.** Das Format beschreibt Kollision und Entities. Grafik-Layer (Tilesets, Parallax, Quads) sind **nicht** Teil davon, sie kommen später (→ O-10, O-16).

## 2. Vorschlag: TOML-Datei mit ASCII-Raster

Die ganze Datei ist gültiges **TOML**. Metadaten stehen als normale Schlüssel darin, das Raster als mehrzeiliger *Literal-String* (`'''…'''`). In einem Literal-String sind alle Zeichen wörtlich, auch `#`, das in TOML sonst einen Kommentar einleitet.

- Dateiendung: **`.emap.toml`**. Editoren erkennen die Datei so als TOML, und sie ist trotzdem eindeutig eine Elora-Karte.
- Ablage: `maps/`
- Parser: Crate `toml` + `serde` in `elora-map`

### Beispiel: Sandbox-Karte (Stand M1; die aktuelle Datei `maps/sandbox.emap.toml` enthält zusätzlich Dummies)

```toml
# Elora-Karte
format  = 1                 # Formatversion
name    = "Sandbox"
author  = "Elora-Team"

# Optional: eigene Zeichen oder Überschreibungen der Standard-Legende
# [legend]
# "~" = "death"

[grid]
tiles = '''
################################################
#..............................................#
#..............................................#
#...a.....................................h....#
#..#####..........%%%%%%%%%%..........#####....#
#..............................................#
#..............................................#
#.......................G......................#
#..................############................#
#..............................................#
#....S..................................S......#
#..######....................................###
#..........%%%%..................%%%%..........#
#..............................................#
#.......................L......................#
#....................#######...................#
#..S..........................................S#
#.....h..........................a.............#
#########^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^#########
################################################
'''
```

## 3. Standard-Legende

### Tiles (Kollision)

| Zeichen | Tile | Bedeutung |
|---|---|---|
| `.` | `air` | Leer |
| `#` | `solid` | Wand, Hook greift |
| `%` | `unhookable` | Wand, Hook greift **nicht** |
| `^` | `death` | Tötet bei Berührung |

### Entities

Eine Entity belegt ihr Feld und macht es zu **Luft**. Sie sitzt in der Mitte des Tiles.

| Zeichen | Entity | Hinweis |
|---|---|---|
| `S` | Spawn (neutral) | DM, LMS, Instagib |
| `R` | Spawn Team Rot | TDM, CTF, LTS |
| `B` | Spawn Team Blau | TDM, CTF, LTS |
| `r` | Flaggenstand Rot | CTF |
| `b` | Flaggenstand Blau | CTF |
| `h` | Herz (Health) | |
| `a` | Rüstung (Armor) | |
| `L` | Laser | E-016 |
| `G` | Granatwerfer | E-016 |
| `D` | Trainings-Dummy, steht | nur Sandbox (E-053, E-054) |
| `W` | Trainings-Dummy, läuft hin und her | nur Sandbox |
| `J` | Trainings-Dummy, springt | nur Sandbox |
| `X` | Trainings-Dummy, läuft + springt | nur Sandbox |

Die Regel für Zeichen: **Satzzeichen sind Tiles, Buchstaben sind Entities.** So bleibt die Legende auch mit neuen Einträgen übersichtlich.

## 4. Regeln

| Regel | Festlegung |
|---|---|
| Koordinaten | Ursprung oben links, x nach rechts, y nach unten. 1 Tile = 32 Einheiten |
| Rastergröße | Ergibt sich aus dem Raster. Alle Zeilen müssen gleich lang sein. Maximum z. B. 1000 × 1000 |
| Leerzeilen | Führende und abschließende Leerzeilen im Raster werden ignoriert |
| Außerhalb der Karte | Gilt als `solid` (niemand fällt aus der Welt) |
| Unbekanntes Zeichen | Fehler mit Zeile und Spalte |
| Pflichtfelder | `format`, `name`, `grid.tiles` |
| Validierung | mindestens 1 Spawn; CTF braucht genau 1× `r` und 1× `b`; Team-Modi brauchen `R` und `B` |
| Unterstützte Modi | werden aus den Entities abgeleitet (z. B. Flaggen vorhanden → CTF möglich) |
| Encoding | UTF-8, Zeilenenden LF oder CRLF |

## 5. Entwicklerkomfort in der Sandbox

- **Hot-Reload:** Die Sandbox beobachtet die Datei und lädt die Karte beim Speichern neu. Elora bleibt dabei an ihrer Position. So lassen sich Karte und Tuning schnell im Wechsel ausprobieren.
- **Fehler** erscheinen im Spiel als Einblendung, statt das Programm zu beenden.

## 6. Betrachtete Alternativen

| Alternative | Warum nicht gewählt |
|---|---|
| Reine ASCII-Datei ohne Kopf | Kein Platz für Name, Version und Legende, also nicht erweiterbar |
| Eigenes INI-artiges Format | Wir müssten einen eigenen Parser schreiben und pflegen, obwohl TOML dasselbe kann |
| JSON mit Zeilen-Array | Das Raster ist schlecht lesbar (Anführungszeichen, Kommas), Kommentare fehlen |
| Tiled (`.tmx`/`.tmj`) | Starker Editor, aber keine handschreibbare Textkarte. Später eventuell als Import (→ O-10) |

## 7. Abgrenzung zum Release-Kartenformat

Dieses Format ist für Release 1 bewusst zu einfach (E-024). Folgendes kann es nicht:

- Grafik-Layer: Tilesets, Deko-Layer vor und hinter dem Spielfeld, Parallax-Hintergründe
- Freie Polygone/Quads, Animationen (Envelopes), Soundquellen
- eingebettete oder referenzierte Bilder, kompakte Speicherung großer Karten
- Anbindung an einen Editor (Rundlauf Laden ↔ Speichern ohne Verlust)

Das Release-Format ist eine eigene Entscheidung (O-10). Das Textformat bleibt daneben für Tests, Golden-Tests und schnelles Prototyping bestehen. Ein Import vom Text- ins Release-Format ist sinnvoll.
