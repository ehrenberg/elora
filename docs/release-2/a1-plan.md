# R2-M1 – Abenteuer-Grundlage: Umsetzungsplan

Status: **Entscheidungen getroffen, Plan zur Freigabe** · Grundlage: E-203 bis E-224, [`weltbuch.md`](weltbuch.md)

## Ziel

Die Technik für das Einzelspieler-Abenteuer – Gegner, NPCs, Dialoge mit Folgen, Aufgaben, Stufen und Fähigkeiten, Ausrüstung, Spielstände, mehrere Karten mit Übergängen – und die Editor-Werkzeuge dafür. Danach können Gebiete wie Karten gebaut werden.

**Abnahme:** Der **Prolog** ist spielbar: Tauwinkel (Grundfassung) und der Weg in die Blütenwiesen bis zum ersten Abschnitt, mit Gesprächen, einer Aufgabe, Gegnern, Beute, Stufenaufstieg und Speichern/Laden (etwa 20–30 Minuten).

## Ausgangslage

- `elora-sim` simuliert Spieler (Mensch, entfernte Figur, Trainings-Dummy), Geschosse, Laser, Pickups und Flaggen deterministisch mit 50 Ticks/s. Dummies sind bereits computergesteuert (`Controller::Dummy`) – dort setzt das Verhalten von Gegnern an.
- Karten (`.emap`) sind in Abschnitte gegliedert; unbekannte Abschnitte werden übersprungen. Ein neuer Abschnitt für Abenteuer-Objekte bricht also nichts.
- Spiel-UI („hell & weich“), Sprachdateien, Editor (egui) und Sandbox stehen.

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| A1.0 ✅ | Entwürfe (E-225) | Oberfläche des Abenteuers (Dialogbox, HUD mit Stufe und Aufgabe, Inventar, Fähigkeitenbaum, Händler) und erste Gegner und NPCs (Stachelkäfer, Pollenbläser, Gras-Hüpfer; Oma Pfütze, Klonk, Lotte, Tüftel, Pip) als Bilder zur Auswahl | Deine Auswahl |
| A1.1 | Fähigkeiten in der Simulation | Hook-Ruck, Heranhooken, Stampfen, Eisgriff, Gleiten als zuschaltbare Fähigkeiten der Figur mit eigenen Tuning-Werten (A-01 ff.); ohne Fähigkeiten ändert sich nichts (Golden-Tests) | Tests + Playtest in der Sandbox |
| A1.2 | Kreaturen | neue Simulations-Elemente für Gegner: Arten mit Verhalten (patrouillieren, springen, fliegen, schießen), Leben, Treffer durch Waffen und Hook, Berührungsschaden, Beute; deterministisch | Tests + Sichtprüfung |
| A1.3 | Abenteuer-Kern | neues Crate `elora-adventure` (reine Logik): Spielstand (Stufe, Erfahrung, Fähigkeitenbaum, Inventar, Glanztropfen, Aufgaben, Weltzustand, Folgen aus Dialogen), Belohnungen, Händler, Ausbau | Tests |
| A1.4 | Dialoge & Aufgaben | Datenformat für Gespräche mit Auswahl, Bedingungen und Folgen sowie für Aufgaben mit Schritten; Texte übersetzbar | Tests |
| A1.5 | Karten-Erweiterung | Abschnitt für Abenteuer-Objekte in `.emap`: Gegner, NPCs (mit Gespräch), Truhen, Schalter, Türen, Sammelstücke, Speicherpunkte, Übergänge zu anderen Karten, Auslöser-Zonen | Tests (Rundweg, kaputte Daten) |
| A1.6 | Abenteuer-Modus | Hauptmenü „Abenteuer“ mit Spielständen; Wechsel zwischen Karten (Dorf ↔ Gebiets-Abschnitte); Tod und Neustart; Speichern | Tests + Sichtprüfung |
| A1.7 | Oberfläche | Dialogbox mit Bild und Auswahl, Sprechblasen für Zurufe, vorausschauende Kamera mit Kamera-Zonen, HUD (Leben, Stufe/Erfahrung, Glanztropfen, aktuelle Aufgabe), Abenteuer-Menü (Inventar, Fähigkeiten, Aufgaben, Karte), Händler und Schmied | Sichtprüfung |
| A1.8 | Editor-Erweiterung | Werkzeug „Abenteuer“: Gegner, NPCs, Objekte, Zonen (Auslöser, Kamera) und Übergänge setzen; Gespräche je NPC anzeigen und direkt testen und einstellen; Testspielen im Abenteuer-Modus | Sichtprüfung |
| A1.9 | Prolog | Tauwinkel (Grundfassung), Tutorial-Weg, erster Abschnitt Blütenwiesen mit Gesprächen, Aufgabe, Gegnern, Beute | Dein Playtest |
| A1.10 | Abnahme | Prolog von Anfang bis Ende spielen, speichern, fortsetzen | Deine Abnahme |

## Technische Festlegungen (Vorschlag)

- **Fähigkeiten und Kreaturen gehören in die Simulation**, nicht in den Client: Der spätere PvP-Modus „Quellenkampf“ (E-204) läuft über das Netz und braucht sie dort.
- **Abenteuer-Logik ist eigenes Crate** (`elora-adventure`): Spielstand, Aufgaben, Dialoge, Belohnungen – testbar ohne Fenster, wie `elora-game` für die Mehrspieler-Regeln.
- **Inhalte als Daten**, nicht als Code: Gegnerarten, Gegenstände, Gespräche und Aufgaben stehen in Dateien unter `assets/adventure/`; Karten verweisen nur auf deren Namen.
- **Grundgefühl bleibt:** Laufen, Springen, Doppelsprung und Hook verhalten sich wie im Mehrspieler (E-212, Weltbuch §6).

## Entscheidungen zu R2-M1

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-A1-01 | Sprache der Abenteuer-Texte | Deutsch und Englisch von Anfang an / zuerst nur Deutsch, Englisch später | **Deutsch + Englisch** (E-217) |
| D-A1-02 | Wo Gespräche und Aufgaben gepflegt werden | im Editor (grafisch) / als Textdateien (lesbar, Git-freundlich) / Textdateien mit Vorschau und Test im Editor | **Textdateien + Editor-Test** (E-218) |
| D-A1-03 | Spielstände | Anzahl der Plätze; nur an Speicherpunkten oder jederzeit; automatisch beim Kartenwechsel | **3 Plätze, automatisch + Speicherpunkte** (E-219) |
| D-A1-04 | Tod im Abenteuer | zurück zum letzten Speicherpunkt ohne Verlust / mit kleinem Verlust (z. B. Glanztropfen) / Wahl je Schwierigkeit | **Speicherpunkt, kleiner Verlust** (E-220) |
| D-A1-05 | Schwierigkeitsgrade | einer / mehrere (z. B. leicht, normal, schwer) | **Eine Stufe** (E-221) |
| D-A1-06 | Darstellung der Gespräche | Textfeld unten mit Bild der Figur / Sprechblasen über den Figuren / beides | **Beides** (E-222) |
| D-A1-07 | Fähigkeiten im normalen Mehrspieler | nur Abenteuer und Rollenspiel-Modus / auch als Server-Option für andere Modi | **Nur Abenteuer + Quellenkampf** (E-223) |
| D-A1-08 | Kamera im Abenteuer | wie im Mehrspieler (fest auf Elora) / leicht vorausschauend und an Räume angepasst | **Vorausschauend + Kamera-Zonen** (E-224) |

## A1.1 Fähigkeiten – Mechanik (E-226 bis E-230)

Die Fähigkeiten sind Schalter an der Figur (`CharacterCore`); ohne sie verhält sich alles wie bisher (Golden-Tests bleiben unverändert). Werte als eigene Tuning-Nummern **A-01 ff.**, in der Sandbox live einstellbar; Ein/Aus je Fähigkeit im Debug-Panel zum Ausprobieren.

| Fähigkeit | Auslöser (Vorschlag) | Wirkung | Tuning (Startwert) |
|---|---|---|---|
| **Hook-Ruck** | Taste **„Fähigkeit“** (neu, Standard: Shift), während der Hook an einer Wand hängt | kräftiger Ruck zum Hook-Punkt, danach normaler Zug | A-01 Ruck-Stärke 16 Einh./Tick · A-02 Abklingzeit 0,8 s |
| **Heranhooken** | Hook trifft Gegenstand, Schalter oder kleinen Gegner | zieht ihn zu Elora (wie Spieler-Hook umgekehrt) | A-03 Zugkraft · kommt mit den Kreaturen in **A1.2** |
| **Stampfen** | **Runter** in der Luft (Druck, nicht Halten) | stößt senkrecht nach unten; beim Aufprall Stoßwelle: betäubt/schadet Gegnern, bricht **Bröckelboden** (neues Tile, bleibt zerbrochen) | A-04 Stampf-Tempo 22 · A-05 Stoßwellen-Radius 64 (2 Tiles) |
| **Eisgriff** | in der Luft **gegen ein Klettertile laufen** (feste, nicht hookbare Wand) | Elora haftet kurz und rutscht langsam; **Springen** stößt von der Wand ab, Doppelsprung bleibt erhalten | A-06 Haftdauer 1,0 s · A-07 Rutschtempo 1,0 · A-08 Wandsprung (9 seitlich, 12 hoch) |
| **Gleiten** | **Springen halten**, wenn Elora fällt | Fallgeschwindigkeit begrenzt, etwas mehr Luftsteuerung | A-09 max. Fallen 2,0 · A-10 Luftsteuerung 7,0 (normal 5,0) |

Technik: zusätzliche Eingabe „Fähigkeit“ in `PlayerInput` (geht erst mit dem Quellenkampf über das Netz, dann Protokoll 7), neue Ereignisse (Ruck, Stampfen, Aufprall, Wandgriff, Wandsprung) für Sound und Effekte, Tiles `Kletterwand` und `Bröckelboden` in Kollision, Karte und Editor.

**Stand A1.1:** umgesetzt bis auf Heranhooken (mit A1.2). Ausprobieren: `cargo run --bin elora -- maps/faehigkeiten-test.emap`, im Debug-Panel (F1) unter „Fähigkeiten (Abenteuer)“ einschalten; Werte unter „Fähigkeiten (A-01 bis A-10)“. Taste „Fähigkeit“ = linke Shift-Taste (in den Einstellungen belegbar). Testkarte: links Halle mit Decke (Hook-Ruck), Bröckel-Brücke über einer Kammer (Stampfen), Kamin und Einzelwand aus Kletterwänden (Eisgriff), vom Sims über die Stachelgrube zur Plattform (Gleiten).
