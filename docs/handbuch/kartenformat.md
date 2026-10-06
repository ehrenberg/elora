# Kartenformat `.emap`

Status: **angenommen** (E-129, E-143 bis E-146) · ersetzt das frühere Textformat `.emap.toml` (E-024, entfernt mit M6.2) · Code: `crates/elora-map`

## 1. Ziele

1. **Ein Format für alles:** Test-, Entwicklungs- und Release-Karten (E-146). Gebaut werden Karten mit dem Editor (ab M6.6).
2. **Kompakt:** zlib-komprimiert (E-143), geeignet für den Download vom Server (E-136).
3. **Robust:** Jede beschädigte oder bösartige Datei führt zu einem Fehler, nie zu einem Absturz. Feste Obergrenzen schützen vor zu großen Daten.
4. **Erweiterbar:** Abschnitte mit Kennung; unbekannte Abschnitte werden übersprungen, die Formatversion steht im Kopf.
5. **Aussehen inklusive:** Materialien, Deko, Hintergrund-Ebenen mit Parallax, Animationen und eingebettete SVGs (E-130 bis E-132, E-144).

## 2. Aufbau der Datei

```
Offset  Inhalt
0       "EMAP"                       Kennung (4 Byte)
4       Formatversion (u16)          zurzeit 1
6       zlib-Strom                   Folge von Abschnitten (entpackt höchstens 32 MiB)
```

Jeder **Abschnitt**: `Kennung (4 Byte ASCII) | Länge (u32) | Inhalt`. Zahlen sind Little Endian, Texte UTF-8 mit vorangestellter Länge (u32), Gleitkommazahlen `f32` und müssen endlich sein. Jeder Abschnitt darf höchstens einmal vorkommen.

| Kennung | Pflicht | Inhalt |
|---|---|---|
| `INFO` | ja | Name, Autor (leer = keiner); je höchstens 128 Byte |
| `GAME` | ja | Breite, Höhe (je 1–1000), dann je Tile 1 Byte Tile-Art (zeilenweise, oben links beginnend) |
| `ENTS` | ja | Anzahl, je Entity: Art (u8), Spalte, Zeile |
| `MATL` | nein | Materialnamen (höchstens 255), dann je Tile 1 Byte: 0 = Standard der Tile-Art, sonst Index + 1 |
| `SKY ` | nein | Himmelsverlauf oben, unten (RGBA); fehlt er, gilt der bisherige Himmel |
| `WTHR` | nein | Wetter (R2-W1): Art (u8: 0 schön, 1 Regen, 2 Gewitter, 3 Nebel, 4 Wind mit Blättern, 5 Wind mit Blüten, 6 Sandsturm, 7 Schnee, 8 Schneesturm), Stärke (f32, 0–1), Wind (f32, −1 bis 1); fehlt er, ist es schön. Nur geschrieben, wenn es Wetter gibt; ältere Programme überspringen ihn |
| `BGRD` | nein | Hintergrund-Ebenen (höchstens 16), von hinten nach vorn: Name, Parallax (x, y), Versatz (x, y), Wiederholung waagerecht (0 = keine), Deko-Liste |
| `DECO` | nein | Deko-Liste hinter der Spielfläche, Deko-Liste davor |
| `ENVL` | nein | Animationen (höchstens 256): Name, Art (0 Bewegung, 1 Farbe), an Server-Zeit gebunden, Punkte (höchstens 1024, Zeit streng aufsteigend): Zeit (ms), 4 Werte, Kurve |
| `IMGS` | nein | Eingebettete SVGs (höchstens 64, je höchstens 512 KiB): Name, Daten |
| `ADVN` | nein | Abenteuer-Objekte (A1.5, höchstens 4096): Id, Position, Art und ihre Werte (siehe unten) |

**Deko-Objekt:** Grafik (0 = eingebaut + Name, 1 = eingebettetes SVG + Index), Position, Skalierung, Drehung (Grad), gespiegelt, Färbung (RGBA), Bewegungs-Animation und Farb-Animation (je Index u16 + Versatz in ms; `0xFFFF` = keine). Insgesamt höchstens 20 000 Deko-Objekte.

**Animationen** laufen in Schleife über die Zeit des letzten Punkts. Bewegung: Versatz x, y (Welteinheiten) und Drehung (Grad). Farbe: r, g, b, a (0 bis 1, multipliziert).

### Kodierung der Aufzählungen

| Code | Tile-Art | | Code | Entity |
|---|---|---|---|---|
| 0 | Luft | | 0 | Spawn (neutral) |
| 1 | Fest | | 1 / 2 | Spawn Rot / Blau |
| 2 | Nicht hookbar | | 3 / 4 | Flaggenstand Rot / Blau |
| 3 | Tod | | 5 / 6 | Herz / Rüstung |
| 4 | Plattform | | 7 / 8 | Laser / Granatwerfer |
| 5 | Eis | | 9–12 | Dummy: steht, läuft, springt, läuft + springt |
| 6 / 7 / 8 | Sprungfeld hoch / schräg links / schräg rechts | | | |
| 9 / 10 | Beschleuniger links / rechts | | | |
| 11 | Kletterwand (E-228) | | | |
| 12 | Bröckelboden (E-230) | | | |

Kurven der Animationen: 0 Stufe, 1 linear, 2 langsam beginnend, 3 schnell beginnend, 4 weich (wie im Original).

### Abenteuer-Objekte (`ADVN`, E-252 bis E-259)

Je Objekt: Id (eindeutig, ohne `:`; Schlüssel im Spielstand), Position (f32 × 2), Art (u8) und deren Werte. Figuren und Gegenstände liegen mit der Mitte, Bereiche und Türen mit der linken oberen Ecke auf `pos`. Eine Karte mit Eingang braucht keinen Mehrspieler-Spawn.

| Code | Art | Werte |
|---|---|---|
| 0 | Gegner | Art aus `creatures.toml`, bleibt besiegt (Boss/besonders, E-235) |
| 1 | NPC | Figur, Gespräch, Blickrichtung, halber Laufweg (0 = steht, E-257) |
| 2 | Truhe | Inhalt (Gegenstand, Anzahl; höchstens 64), Schloss-Bedingung (leer = offen, E-255) |
| 3 | Schalter | Merker, nur einmal, Auslöser: Aktionstaste / Hammer / Hook (E-256) |
| 4 | Tür | Größe in Tiles (auf dem Raster), Bedingung zum Öffnen (E-254) |
| 5 | Sammelstück | Gegenstand |
| 6 | Speicherpunkt | – |
| 7 | Heilpflanze | Leben (E-258) |
| 8 | Eingang | – (Ziel von Übergängen) |
| 9 | Übergang | Größe, Zielkarte, Ziel-Eingang, beim Hineinlaufen (sonst Aktionstaste, E-252) |
| 10 | Zone | Größe (für Aufgaben „Ort erreichen“) |
| 11 | Kamera | Größe, Art: festsetzen / begrenzen (E-259) |

Die Karte prüft den Aufbau (Ids, Lage, Größen, Raster); Verweise auf Gegnerarten, Figuren, Gespräche, Gegenstände, Bedingungen und Zielkarten prüft `elora-adventure` (`check::map_objects`, `check::map_links`).

## 3. Bedeutung der Tile-Arten

| Tile | Bedeutung |
|---|---|
| Luft | Leer |
| Fest | Wand, Hook greift |
| Nicht hookbar | Wand, Hook greift **nicht** |
| Tod | Tötet bei Berührung |
| Plattform | Trägt von oben, von unten/seitlich durchlässig; Hook, Granate und Laser fliegen hindurch; mit „Runter“ fällt man hindurch (T-36, E-141) |
| Eis | Wand, rutschig (T-31, T-32) |
| Sprungfeld | Wirft eine darauf stehende Figur hoch oder schräg (T-33, T-34); Hook greift |
| Beschleuniger | Trägt eine darauf stehende Figur wie ein Laufband (T-35); Hook greift |
| Kletterwand | Wand, Hook greift **nicht**; mit Eisgriff kann Elora daran haften und abspringen (E-228) |
| Bröckelboden | Wand, Hook greift; bricht beim Stampfen und bleibt zerbrochen (E-230) |

## 4. Regeln

| Regel | Festlegung |
|---|---|
| Koordinaten | Ursprung oben links, x nach rechts, y nach unten. 1 Tile = 32 Einheiten. Entities sitzen in der Tile-Mitte |
| Außerhalb der Karte | gilt als fest (niemand fällt aus der Welt) |
| Validierung | mindestens 1 Spawn; Flaggen nur als Paar (genau 1× Rot und 1× Blau); Entities innerhalb des Rasters; Verweise auf Bilder und Animationen (der passenden Art) müssen existieren |
| Unterstützte Modi | aus den Entities abgeleitet: neutrale Spawns → DM/LMS/Instagib, rote + blaue Spawns → TDM/LTS, dazu ein Flaggenpaar → CTF |
| Prüfsumme | BLAKE2s-256 über die Datei-Bytes; identifiziert die Karte beim Download und im Zwischenspeicher (M6.5) |
| Eingebettete SVGs | Die Karte prüft nur Anzahl und Größe. Der Client parst sie beim Zeichnen **ohne externe Verweise** (keine Dateien, keine Netzadressen, M6.4) |

## 5. Werkzeuge

- **Ansehen:** `cargo xtask map-dump maps/<karte>.emap` gibt Kopf, Prüfsumme, Modi, Ebenen und das Raster als Zeichen aus.
- **Tests:** `Map::from_rows` baut Karten aus Zeichenrastern (Tiles wie in den Aufzeichnungen: `. # % ^ = ~ ! \ / < >`; Entities `S R B r b h a L G D W J X`). Das ist eine Hilfe im Code, kein Dateiformat.
- **Hot-Reload:** Die Sandbox beobachtet die Kartendatei und lädt sie beim Speichern neu (z. B. aus dem Editor).

## 6. Eingebaute Grafik (Stil A, M6.3)

Ablage `assets/map/`, Übersicht in [`../archiv/release-1/design/elora-kartenteile.png`](../archiv/release-1/design/elora-kartenteile.png). Erzeugt einmalig mit `tools/../archiv/release-1/design/elora_map_assets.py`, danach normale, von Hand änderbare SVGs.

| Art | Namen | Hinweis |
|---|---|---|
| Materialien (`MATL`) | `earth`, `sand`, `snow` (fest) · `stone` (nicht hookbar) · `ice` (Eis) | Farben, Rundung und Detail-Anteil in `materials.toml`; je Material ein SVG mit Kappen (`cap`, `cap-left`, `cap-right`, `cap-single`) und Details (`detail-1` …). Das erste Material einer Tile-Art ist ihr Standard |
| Spezial-Tiles | `tiles/death`, `platform`, `jump`, `conveyor` | fest zugeordnet; Stacheln zeigen vom Untergrund weg, Sprungfeld links und Beschleuniger links sind gespiegelt |
| Deko (`Art::Builtin`) | `bush-1`, `bush-2`, `flower-pink`, `flower-yellow`, `flower-blue`, `grass-1`, `grass-2`, `rock-1`, `rock-2`, `mushroom-red`, `mushroom-brown`, `tree-round`, `tree-pine`, `fence`, `sign-arrow`, `sign-board` | Ursprung unten in der Mitte |
| Hintergrund (`Art::Builtin`) | `cloud-1` … `cloud-3`, `moon` (Ursprung Mitte) · `hills-far`, `hills-near`, `mountains`, `forest` (Ursprung unten links) · `stars` (oben links) | Streifen 1024 breit und nahtlos wiederholbar; bei Nacht über die Färbung abgedunkelt |

**Auto-Kanten:** Feste Tiles bilden mit Sprungfeldern und Beschleunigern eine Fläche. Außenecken mit zwei freien Nachbarn werden gerundet, die Kontur liegt nur an freien Kanten, Kappen an jeder freien Oberkante (mit Endstück an freien Seiten), Details fest je Tile verstreut.

## 7. Darstellung im Spiel (M6.4)

- **Reihenfolge:** Himmel → Hintergrund-Ebenen (hinten nach vorn) → Deko hinten → Spielfläche → Figuren, Items, Geschosse → Deko vorn → Effekte.
- **Parallax:** Ein Objekt einer Hintergrund-Ebene liegt bei `Position + Versatz + Kamera-Mitte × (1 − Parallax)`. Parallax 1 bewegt sich mit der Spielfläche, 0 steht fest im Bild. Mit Wiederholung wird die Ebene waagerecht über den ganzen Ausschnitt gelegt.
- **Animationen:** Bewegung wirkt als Versatz (x, y) und Drehung, Farbe wird multipliziert. An die Server-Zeit gebundene Animationen sehen alle Spieler in derselben Phase (z. B. ziehende Wolken), die anderen laufen nach der Uhr des Clients.
- **Zwischenspeicher:** Die Spielfläche wird in Stücken von 16 × 16 Tiles einmal tesselliert und nur bei Änderungen der Karte neu gebaut; nur sichtbare Stücke werden gebaut und gezeichnet. Die Pfeile der Beschleuniger laufen je Frame mit.
- **Eingebettete SVGs** lädt der Client ohne jede Auflösung externer Verweise und mit höchstens 200 000 Ecken je Bild; ungültige Bilder bleiben unsichtbar.
- **Vorführkarte:** `maps/look-test.emap` (erzeugt aus `map_view.rs`, Test `write_look_test_map`).

## 8. Übertragung (M6.5, E-136)

1. Client → `Join`. Der Server antwortet mit `MapInfo` (Name, Prüfsumme, Größe) – der Spieler ist **noch nicht** in der Welt.
2. Der Client sucht die Karte mit genau dieser Prüfsumme: zuerst unter den Downloads (`~/.local/share/elora/downloads/<name>-<prüfsumme>.emap`, unter Windows/macOS im Einstellungsordner), dann in `maps/<name>.emap`.
3. Fehlt sie, fordert er sie mit `MapRequest` in Teilen zu 16 KiB an (4 Teile gleichzeitig unterwegs) und zeigt den Fortschritt. Nach dem letzten Teil prüft er Größe und Prüfsumme und legt die Datei ab.
4. Client → `MapReady`. Erst jetzt bekommt er einen Slot und `Welcome` (mit der Prüfsumme zur Kontrolle).
5. **Kartenwechsel:** Der Server schickt allen erneut `MapInfo`; alle laden und treten neu bei.

Grenzen: Karten bis 4 MiB (größere lädt der Server gar nicht erst), jeder Teil höchstens zweimal je Client; Dateinamen aus Servernamen werden auf `A–Z a–z 0–9 - _` gekürzt. Protokollversion 5.
