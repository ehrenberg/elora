# R2-M2.1 – Kapitel 1: Blütenwiesen – Umsetzungsplan

Status: **Entscheidungen getroffen (E-297 bis E-302), Plan zur Freigabe** · Grundlage: [`weltbuch.md`](weltbuch.md) §4.1 und §5, [`prolog.md`](prolog.md), E-272 bis E-295

## Ziel

Das erste Kapitel ist von Anfang bis Ende spielbar: nach dem Prolog weiter durch die Blütenwiesen, Imkerin Wabe und ihre Bienen, der Kampf gegen den Hüter **Brummbär-Hummel**, der **Quellfunke** für Tüftel und die neue Fähigkeit **Hook-Ruck**, dazu ein Dorf, das wieder etwas Farbe bekommt.

**Abnahme:** Kapitel 1 einmal durchspielen (etwa 60–90 Minuten mit Nebenaufgaben), dazwischen speichern und fortsetzen; danach steht „Die Blütenquelle“ als erledigt im Aufgabenbuch und das nächste Kapitel ist angekündigt.

## Ausgangslage (aus R2-M1)

- Gegner mit Verhalten Läufer, Hüpfer, Schütze, Flieger; Arten mit `boss = true` bleiben besiegt (E-235), haben aber noch **kein eigenes Verhalten und keine Lebensleiste im HUD**.
- Fähigkeiten in der Simulation fertig (A1.1); freigeschaltet über die Folge `faehigkeit hook-ruck`.
- Farbrückkehr im Dorf vorbereitet: Merker `quellen_befreit` macht `-blass`-Deko teilweise bunt (E-277).
- Karten entstehen als Generator in `apps/elora-client/src/editor/prolog.rs` (Gelände, Objekte, Deko), Grafiken über die Python-Skripte (E-295), Musik je Gebiet über `worldmap.toml` (E-290).
- `wiese-1` endet am Wiesenrand mit Quellstein; dort beginnt „Die Blütenquelle“ (Schritt „Tiefer in die Blütenwiesen“).

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| M2.1.0 | Entwürfe ([`design/kapitel1-entwuerfe.png`](design/kapitel1-entwuerfe.png)) | Imkerin Wabe (Figur und Bild im Gespräch), Brummbär-Hummel (Ruhe, Flug, Sturzflug, betäubt, besiegt), verwirrte Biene (Gegner der Phase 3), Biene (Sammelstück), Wabenhut, Girlanden und Festlaternen, Hook-Blüte (Hookpunkt in der Arena), Quellfunke (Gegenstand), Bienenstöcke und Imkerei als Deko, Blütenquelle (befreit/verdorrt) | Deine Auswahl |
| M2.1.1 | Hüter-Technik | neues Verhalten **Boss** in der Simulation mit Phasen (Muster aus Schritten: kreisen, anvisieren, Sturzflug, betäubt am Boden), verwundbar nur in bestimmten Phasen; Lebensleiste mit Namen oben im HUD; Arena: Ein- und Ausgang schließen sich beim Kampf (Tür mit Bedingung, gibt es schon), Kamera-Zone „Festsetzen“; nach dem Sieg Ereignis für Aufgaben und Merker | Tests + Sandbox (F1 → Gegner) |
| M2.1.2 | Hookpunkte | **Hook-Blüte**: einzelner Hookpunkt mitten in der Luft (Objekt der Karte, für die Simulation ein hookbares Tile ohne Kollision für Figuren), damit Elora in der Arena oben bleiben kann | Tests + Sichtprüfung |
| M2.1.3 | Inhalte | Figur und Gespräch **Wabe**; Nebenaufgabe **„Wabes Bienen“** (fünf Bienen in den Abschnitten, teils hinter Hook-Stellen); Hauptaufgabe **„Die Blütenquelle“** in Schritten bis zum Quellfunken; neue Gespräche für Oma, Tüftel, Pip, Klonk und Lotte nach Kapitel 1; Zurufe im Dorf ändern sich | Tests (Inhaltsprüfung, Durchlauf) |
| M2.1.4 | Karten | `wiese-2` (Wälder aus Riesenblumen, Imkerei, Bach, erste schwere Hook-Strecke), `wiese-3` (Höhlen unter den Wurzeln, Dornen, versteckte Biene, Weg nach oben zur Arena), `wiese-arena` (Blütenquelle, Hook-Blüten, Arena) – Größe und Deko wie `wiese-1`; Rückwege zu den früheren Abschnitten | Sichtprüfung + Durchlauf-Test |
| M2.1.5 | Quellfunke und Hook-Ruck | Sieg → Quellfunke; bei Tüftel abgeben → Fähigkeit **Hook-Ruck** (Folge `faehigkeit hook-ruck`) mit Erklärung per Wegweiser-Text; erste Stelle, die nur mit Hook-Ruck geht (Rückkehr nach `wiese-1` lohnt sich: verstecktes Sammelstück) | Tests |
| M2.1.6 | Dorf nach Kapitel 1 | Merker `quellen_befreit = 1`: Teil der Beete, Blumenkästen und Fahnen bunt; Fest-Moment am Brunnen (kurzes Gespräch mit allen); Weltkarte zeigt die Blütenwiesen als befreit | Sichtprüfung |
| M2.1.7 | Musik und Sounds | Boss-Musik (CC0 oder CC BY, zum Anhören vorgelegt, E-285); Sounds für Hummel (Brummen, Sturzflug, Treffer), Biene (Sammeln), Quellfunke (Fanfare) aus Kenney oder anderen freien Quellen | Deine Hörprobe |
| M2.1.8 | Abnahme | Kapitel 1 durchspielen, speichern, fortsetzen | Deine Abnahme |

## Ablauf von Kapitel 1 (Vorschlag)

| # | Ort | Was passiert |
|---|---|---|
| 1 | `wiese-1`, Wiesenrand | Quellstein; Aufgabe „Die Blütenquelle“: tiefer in die Wiesen |
| 2 | `wiese-2` | **Imkerin Wabe** an ihrer Imkerei: die Bienen sind ausgeschwärmt, weil die Hummel tobt; Nebenaufgabe „Wabes Bienen“ (1 Biene liegt hier) |
| 3 | `wiese-2` | Bach und Riesenblumen, Hook-Strecke; Gegner wie bisher, dazu mehr Pollenbläser |
| 4 | `wiese-3` | Höhlen unter den Wurzeln, Dornen; Quellstein vor dem Aufstieg; 2 Bienen |
| 5 | `wiese-arena` | Blütenquelle verdorrt, die **Brummbär-Hummel** kreist darüber; Kampf |
| 6 | `wiese-arena` | Sieg: Die Hummel landet erschöpft und war nur verwirrt (Gespräch); Quellfunke, die Quelle blüht auf |
| 7 | Tauwinkel | Tüftel baut den **Hook-Ruck** und öffnet den neuen Teil seines Hofs; **kleine Feier** am Brunnen (Laternen, Girlanden, Festmusik); Oma erzählt von der zweiten Quelle (Ankündigung Murmelwald) |
| 8 | frei | restliche Bienen (2 davon nur mit Hook-Ruck) zu Wabe bringen: **Wabenhut** und Erfahrung |

## Der Kampf gegen die Brummbär-Hummel (Vorschlag)

- **Phase 1 – Kreisen:** Die Hummel fliegt hoch über der Arena Kreise und lässt ab und zu Pollen fallen (wie Pollenbläser-Kugeln). Elora hält sich mit dem Hook an den **Hook-Blüten** oben.
- **Phase 2 – Sturzflug:** Sie visiert Elora an (kurze Warnung: Brummen wird lauter, Schatten am Boden) und stürzt herab. Weicht Elora aus, bleibt die Hummel **kurz benommen am Boden** – jetzt trifft der Hammer voll, andere Waffen halb.
- **Phase 2b – ab halbem Leben:** schneller, zwei Sturzflüge hintereinander, die Hook-Blüten welken abwechselnd.
- **Phase 3 – ab einem Drittel Leben:** Sie ruft **kleine, verwirrte Bienen**, die Elora umschwirren und leicht schaden (besiegbar). Nach dem Kampf fliegen sie zu Wabe zurück.
- **Treffer nur, solange sie benommen ist** – in der Luft prallen sie ab (kleine Sternchen), damit das Ausweichen zählt (E-299).
- Elora stirbt? Zurück zum Quellstein vor der Arena (E-261), die Hummel hat wieder volles Leben.

## Entscheidungen zu R2-M2.1

| # | Frage | Entscheidung |
|---|---|---|
| D-M21-01 | Zahl der neuen Abschnitte | **Drei:** `wiese-2`, `wiese-3`, `wiese-arena` (E-297) |
| D-M21-02 | Kampf gegen die Hummel | **Schwerer:** Kreisen mit Pollen, Sturzflug, benommen; ab halbem Leben schneller mit welkenden Hook-Blüten; dritte Phase mit gerufenen Bienen (E-298) |
| D-M21-03 | Wie die Hummel verwundbar ist | **Nur benommen am Boden** (E-299) |
| D-M21-04 | Belohnung für die Bienen | **Ausrüstung:** Wabenhut mit kleinem Bonus, dazu Erfahrung (E-300) |
| D-M21-05 | Fest im Dorf nach Kapitel 1 | **Kleine Feier:** Gespräch am Brunnen, Laternen, Girlanden und Festmusik, bis Elora weiterzieht (E-301) |
| D-M21-06 | Hook-Ruck erklären | **Übungsplatz im Hof:** Tüftel erklärt und öffnet einen neuen Teil seines Hofs (E-302) |

## Technische Festlegungen (Vorschlag)

- **Boss-Verhalten in der Simulation** (deterministisch, wie alle Gegner), Phasen und Werte als Daten in `creatures.toml`; der Client zeigt nur an.
- **Hook-Blüte** als neues Tile „Hookpunkt“ (hookbar, für Figuren durchlässig) statt Objekt – dann gilt es auch im Mehrspieler-Editor; Darstellung als Blüte über die Materialien.
- **Karten weiter aus Generatoren** wie Tauwinkel und `wiese-1`; Sammelstücke, Gegner und Gespräche als Objekte.
- Durchlauf-Test `tests/prolog.rs` wird um Kapitel 1 erweitert (Gespräche, Aufgaben, Übergänge, Boss-Sieg über Ereignisse, Speichern/Fortsetzen).
