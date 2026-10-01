# M6 – Karten & Editor: Umsetzungsplan

Status: **angenommen** (E-129–E-138), in Umsetzung · Grundlage: [`06-roadmap.md`](06-roadmap.md) M6 (nach M7, E-111), E-024, E-028, E-030, E-031, E-089, O-37, O-43

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

## Arbeitsschritte (nach den Entscheidungen)

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M6.0 | Entwürfe | – | Kartenlook als Bild zur Auswahl: Materialien mit Kanten/Ecken, Deko, Hintergrund-Ebenen (E-130) | Deine Auswahl |
| M6.1 | Neue Tile-Arten | `elora-sim` | Plattform, Eis, Sprungfeld, Beschleuniger (E-137) in Kollision und Bewegung; Tuning-Vorschlag T-31 ff. zur Freigabe; Testkarte; Golden-Tests der alten Karten unverändert | Tests + dein Playtest · **abgeschlossen** (E-140–E-142; `maps/tiles-test.emap.toml`) |
| M6.2 | Release-Format | `elora-map` | Binäres Datenmodell (E-129): Kopf mit Version, Abschnitte (Game-Layer mit Tile-Arten und Richtungen, Material-Layer, Deko, Hintergrund-Ebenen mit Parallax, Envelopes, Metadaten), komprimiert, Prüfsumme; Import der Textkarten | Tests (Rundweg, Import, kaputte Dateien) |
| M6.3 | Materialien & Deko | `assets/` | SVG-Sätze je Material mit Kanten/Ecken (Auto-Kanten-Regeln), Deko-Objekte, Hintergründe – nach dem Entwurf aus M6.0 | Sichtprüfung |
| M6.4 | Karten-Darstellung | Client, `elora-render` | Ebenen zeichnen (Parallax, Deko vor/hinter der Spielfläche), Auto-Kanten, Envelopes abspielen, gecachte Meshes | Sichtprüfung, Benchmark |
| M6.5 | Übertragung | Protokoll, Server, Client | Karte komprimiert in Teilen, Prüfsumme, Zwischenspeicher im Client (E-136) | Integrationstest |
| M6.6 | Editor-Grundlage | Client (`editor/`, egui) | Editor aus dem Hauptmenü, Kamera, Raster, Ebenen-Liste, Neu/Laden/Speichern, Rückgängig/Wiederholen | Tests + Sichtprüfung |
| M6.7 | Werkzeuge | Client | Pinsel, Rechteck, Füllen, Radierer, Tile-Arten mit Richtung, Materialien, Entities (Spawns, Pickups, Flaggen, Dummies), Auswahl kopieren | Sichtprüfung |
| M6.8 | Deko, Hintergrund, Animation | Client | Deko platzieren/drehen/skalieren, Hintergrund-Ebenen mit Parallax, Envelope-Editor (Kurven für Position, Drehung, Farbe) (E-133) | Sichtprüfung |
| M6.9 | Testspielen | Client | aus dem Editor direkt in eine Trainingsrunde und zurück | Sichtprüfung |
| M6.10 | Release-Karten | `maps/` | 3 DM + 2 CTF nach deinen Vorgaben (E-134/E-135) | Dein Playtest |
| M6.11 | Abnahme | – | Karte von Grund auf bauen und spielen | Deine Abnahme |

## Entscheidungen zu M6

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-M6-01 | Speicherformat (O-37) | **Text** (TOML, lesbar, Git-freundlich, größer) / **Binär** (kompakt, wie Original) / **Text, beim Übertragen komprimiert** | E-129: **binär** |
| D-M6-02 | Grafik der Kartenteile | **Vektor-Kacheln mit automatischen Kanten/Ecken** (Tile-Raster bleibt, Look aus SVG-Sätzen je Material) / **freie Vektorformen** (Polygone unabhängig vom Raster) / **beides** (Raster für Kollision, freie Formen als Deko) | E-130: **beides** – Kacheln mit Auto-Kanten + freie Vektor-Deko |
| D-M6-03 | Ebenen-Modell | wie Original (Gruppen mit Parallax, Tile- und Quad-Layer) / vereinfacht (Game-Layer + Deko-Layer davor/dahinter + Hintergrund-Ebenen mit Parallax) | E-131: **vereinfacht** |
| D-M6-04 | Animationen | ja (bewegte Deko, Farbwechsel – wie Envelopes) / nein für Release 1 | E-132: **wie Original** (Envelopes) |
| D-M6-05 | Editor-Umfang Release 1 | Grundwerkzeuge (Pinsel, Rechteck, Füllen, Entities, Layer, Rückgängig, Testspielen) / zusätzlich Deko & Hintergrund / zusätzlich Animationen | E-133: **alles** inkl. Animations-Editor |
| D-M6-06 | Release-Karten (O-43) | Anzahl und Modi (z. B. 3 DM + 2 CTF), wer sie baut (ich mit dem Editor nach deinen Vorgaben / du / gemeinsam) | E-134/E-135: **3 DM + 2 CTF**, gebaut von Claude nach deinen Vorgaben |
| D-M6-07 | Karten-Download | Server schickt fehlende Karten automatisch (wie Original) / Karten müssen vorher installiert sein | E-136: **automatisch** |
| D-M6-08 | Neue Tile-Arten | nur die bisherigen (fest, Tod, nicht hookbar) / zusätzliche (z. B. Plattform von unten durchlässig) | E-137: **Plattform, Eis, Sprungfeld, Beschleuniger** |

## Technische Festlegungen (Vorschlag)

- **Kollision bleibt ein Raster** aus 32er-Tiles (Physik und Netzcode unverändert); die Grafik legt sich darüber.
- **Rückwärtskompatibel:** Textkarten (`.emap.toml`) bleiben lesbar und werden beim Laden ins neue Modell überführt; Server und Client verstehen beide.
- **Prüfsumme** (BLAKE2s, schon über `snow` vorhanden) identifiziert Karten beim Download und im Zwischenspeicher.
- **Editor in egui** (E-031) – Werkzeugleisten und Listen sind dort schnell gebaut; die Karte selbst zeichnet der Vektor-Renderer.
