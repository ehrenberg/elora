# M6 – Karten & Editor: Umsetzungsplan

Status: **Entwurf, Entscheidungen offen** · Grundlage: [`06-roadmap.md`](06-roadmap.md) M6 (nach M7, E-111), E-024, E-028, E-030, E-031, E-089, O-37, O-43

## Ziel

Ein eigenes Release-Kartenformat mit Grafik und ein ins Spiel integrierter Editor (E-028): Karten lassen sich von Grund auf bauen, direkt aus dem Editor testen und auf Servern spielen. Dazu die ersten echten Karten für Release 1.

**Abnahme (Roadmap):** Mit dem Editor lässt sich eine Karte von Grund auf bauen und spielen.

## Was das Original macht (Teeworlds 0.7, `datafile.cpp`, `mapitems.h`, `editor/`)

| Bereich | Original |
|---|---|
| **Datei** | Binäres „Datafile“ (`.map`): Einträge (Items) + mit zlib komprimierte Datenblöcke; Bilder und Sounds können eingebettet sein |
| **Gruppen** | Ebenen-Gruppen mit **Parallax** (x/y in %), Versatz, optionalem Ausschnitt (Clipping) |
| **Layer** | **Game-Layer** (Kollision: leer, fest, Tod, nicht hookbar + Entities als Tile-Indizes), **Tile-Layer** (Bild aus 16×16 Kacheln, je Tile Index + Spiegelung/Drehung), **Quad-Layer** (freie, texturierte Vierecke mit Farbe je Ecke), **Sound-Layer** (0.7) |
| **Animation** | **Envelopes**: Kurven für Position, Drehung und Farbe; Quads und Tile-Layer-Farbe hängen daran (z. B. ziehende Wolken, pulsierendes Licht) |
| **Automapper** | Regeln wählen passende Rand-/Ecken-Kacheln automatisch aus den Nachbarn |
| **Editor** | Im Client (Taste/Menü), Pinsel aus Kacheln, Rechteck, Füllen, Kachel-Auswahl, Layer und Gruppen verwalten, Quads bearbeiten, Envelope-Editor, Bilder/Sounds einbetten, Testspielen über lokalen Server |
| **Übertragung** | Server schickt die Karte in Teilen an Clients, die sie nicht haben (Prüfsumme + Name); Clients speichern sie im Download-Ordner |
| **Vanilla-Karten** | dm1, dm2, dm6, dm7, dm8, dm9, ctf1–ctf7 – kleine bis mittlere Arenen, klar lesbare Kollision |

## Stand in Elora

- **Textformat** (E-024, `.emap.toml`): nur Kollision und Entities als ASCII-Raster – für Test- und Entwicklungskarten. Zwei Karten: `sandbox`, `ctf-test`.
- **Welt-Optik** (E-089): Tiles einfarbig mit Kontur, Himmel als Verlauf – schlicht, ohne Grafik-Layer.
- **Übertragung:** Der Server schickt die Karte als Text im `Welcome` (bis 4 MB, ein Paket über den zuverlässigen Kanal).
- **Stil:** Alles ist Vektor (E-030), Assets sind SVG (M5.2) – Figur, Items, Emotes.
- **UI:** egui ist für den Editor vorgesehen (E-031); die Spiel-UI hat ein eigenes Toolkit (M7.1).

## Arbeitsschritte (Vorschlag)

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M6.0 | Entwürfe | – | Kartenlook als Bild zur Auswahl (Kanten, Ecken, Hintergrund-Ebenen, Deko) – D-M6-02 | Deine Auswahl |
| M6.1 | Release-Format | `elora-map` | Datenmodell (D-M6-01/03), Lesen/Schreiben, Versionsnummer, Prüfungen, Import der Textkarten | Tests (Rundweg, Import) |
| M6.2 | Karten-Darstellung | Client, `elora-render` | Layer zeichnen: Parallax, Kanten/Ecken (D-M6-02), Deko, Animationen (D-M6-04), gecachte Meshes | Sichtprüfung, Benchmark |
| M6.3 | Übertragung | Protokoll, Server, Client | Karte komprimiert und in Teilen, Prüfsumme, Zwischenspeicher im Client (D-M6-07) | Integrationstest |
| M6.4 | Editor-Grundlage | Client (`editor/`, egui) | Editor-Modus aus dem Hauptmenü, Kamera, Raster, Layer-Liste, Speichern/Laden, Rückgängig/Wiederholen | Tests + Sichtprüfung |
| M6.5 | Werkzeuge | Client | Pinsel, Rechteck, Füllen, Radierer, Entities setzen (Spawns, Pickups, Flaggen, Dummies), Auswahl kopieren | Sichtprüfung |
| M6.6 | Grafik-Werkzeuge | Client | je nach D-M6-02/03/04: Kanten automatisch, Deko platzieren, Hintergrund-Ebenen, Animationen | Sichtprüfung |
| M6.7 | Testspielen | Client | aus dem Editor direkt in eine Trainingsrunde mit der Karte und zurück (Entwurf bleibt erhalten) | Sichtprüfung |
| M6.8 | Release-Karten | `maps/` | Karten nach D-M6-06 bauen (mit dem Editor) | Playtest |
| M6.9 | Abnahme | – | Karte von Grund auf bauen und spielen | Deine Abnahme |

## Entscheidungen zu M6

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-M6-01 | Speicherformat (O-37) | **Text** (TOML, lesbar, Git-freundlich, größer) / **Binär** (kompakt, wie Original) / **Text, beim Übertragen komprimiert** | offen |
| D-M6-02 | Grafik der Kartenteile | **Vektor-Kacheln mit automatischen Kanten/Ecken** (Tile-Raster bleibt, Look aus SVG-Sätzen je Material) / **freie Vektorformen** (Polygone unabhängig vom Raster) / **beides** (Raster für Kollision, freie Formen als Deko) | offen |
| D-M6-03 | Ebenen-Modell | wie Original (Gruppen mit Parallax, Tile- und Quad-Layer) / vereinfacht (Game-Layer + Deko-Layer davor/dahinter + Hintergrund-Ebenen mit Parallax) | offen |
| D-M6-04 | Animationen | ja (bewegte Deko, Farbwechsel – wie Envelopes) / nein für Release 1 | offen |
| D-M6-05 | Editor-Umfang Release 1 | Grundwerkzeuge (Pinsel, Rechteck, Füllen, Entities, Layer, Rückgängig, Testspielen) / zusätzlich Deko & Hintergrund / zusätzlich Animationen | offen |
| D-M6-06 | Release-Karten (O-43) | Anzahl und Modi (z. B. 3 DM + 2 CTF), wer sie baut (ich mit dem Editor nach deinen Vorgaben / du / gemeinsam) | offen |
| D-M6-07 | Karten-Download | Server schickt fehlende Karten automatisch (wie Original) / Karten müssen vorher installiert sein | offen |
| D-M6-08 | Neue Tile-Arten | nur die bisherigen (fest, Tod, nicht hookbar) / zusätzliche (z. B. Plattform von unten durchlässig) | offen |

## Technische Festlegungen (Vorschlag)

- **Kollision bleibt ein Raster** aus 32er-Tiles (Physik und Netzcode unverändert); die Grafik legt sich darüber.
- **Rückwärtskompatibel:** Textkarten (`.emap.toml`) bleiben lesbar und werden beim Laden ins neue Modell überführt; Server und Client verstehen beide.
- **Prüfsumme** (BLAKE2s, schon über `snow` vorhanden) identifiziert Karten beim Download und im Zwischenspeicher.
- **Editor in egui** (E-031) – Werkzeugleisten und Listen sind dort schnell gebaut; die Karte selbst zeichnet der Vektor-Renderer.
