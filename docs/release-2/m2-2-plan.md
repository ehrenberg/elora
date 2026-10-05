# R2-M2.2 – Kapitel 2: Murmelwald – Umsetzungsplan

Status: **Freigegeben, in Umsetzung** (Entscheidungen E-306 bis E-311) · Grundlage: [`weltbuch.md`](weltbuch.md) §4.2 und §5, [`m2-1-plan.md`](m2-1-plan.md), E-296, E-305

## Ziel

Kapitel 2 ist von Anfang bis Ende spielbar: aus Tauwinkel in den Murmelwald, der alte Uhu **Plumm**, **Erinnerungsrunen**, die von einer sechsten Quelle erzählen, das verirrte **Pilzkind**, der Hüter **Wurzelwächter** und die neue Fähigkeit **Heranhooken**. Danach wird Tauwinkel noch etwas bunter (`quellen_befreit = 2`).

**Abnahme:** Kapitel 2 einmal durchspielen (etwa 60–90 Minuten mit Nebenaufgaben), dazwischen speichern und fortsetzen.

## Ausgangslage (aus R2-M1 und R2-M2.1)

- Gegner-Verhalten: Läufer, Hüpfer, Schütze, Flieger, Hüter aus der Luft (`diver`); Hüter-Lebensleiste, Merker `besiegt.<art>`, Ereignisse für Klänge.
- Fähigkeit **Heranhooken** (`Pull`) gibt es in der Simulation schon (A1.2): Hook zieht kleine Gegner zu Elora. Für Kapitel 2 fehlt noch das Ziehen von **Gegenständen und Schaltern**.
- Bausteine aus Kapitel 1: Karten-Generatoren, Hook-Blüten, Ruck-Stellen, Deko mit Zustand (`-verdorrt`/`-befreit`, `-fest`), Figuren mit `show_if`, Musik je Gebiet, Kampfmusik.
- Nach Kapitel 1 läuft die Aufgabe „Das Flüstern im Murmelwald“ (Ankündigung, Schritt von Hand).

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| M2.2.0 ✅ | Entwürfe ([`design/kapitel2-entwuerfe.png`](design/kapitel2-entwuerfe.png), E-312) | Uhu Plumm, Pilzkind, Wurzelschlange, Eichhornpirat (mit Nuss), Pilzwicht, Wurzelwächter (Ruhe, Wurzelangriff, Kern offen, besiegt), Erinnerungsrune, Kern, Waldquelle (verdorrt/befreit), Deko des Waldes (hohe Bäume, Baumhäuser, Hängebrücken, leuchtende Pilze, Wurzeln, Moos) | Deine Auswahl |
| M2.2.1 ✅ | Neue Gegner | **Wurzelschlange**: versteckt im Boden, schießt hoch, wenn Elora nah ist, zieht sich zurück (verwundbar nur draußen). **Eichhornpirat**: sitzt auf Ästen, wirft Nüsse im Bogen. **Pilzwicht**: läuft, bei Berührung ein bunter Rausch (Elora regenbogenfarben und langsamer, E-311); **Begleiter** für das Pilzkind (folgt Elora, wartet an schwierigen Stellen, E-308) | Tests + Sandbox |
| M2.2.2 ✅ | Heranhooken erweitern | Hook zieht **Gegenstände** (Truhen-Inhalt, Kerne, Sammelstücke) und **Hook-Schalter** (Hebel, die man nur mit Heranhooken umlegt) zu Elora; neue Schalter-Art „Zugschalter“ in Karte und Editor | Tests |
| M2.2.3 | Hüter-Technik | **Wurzelwächter**: großer Hüter am Boden, verschließt Wege mit Wurzeln (zeitweise feste Tiles), schlägt mit Wurzeln aus dem Boden; **Kerne** in seiner Rinde: Hook daran und wegziehen (Tauziehen) legt einen Kern frei, dann verwundbar | Tests + Sandbox |
| M2.2.4 | Inhalte | Figuren Plumm und Pilzkind; Hauptaufgabe „Das Flüstern im Murmelwald“ bis zum Quellfunken; Nebenaufgaben **„Erinnerungsrunen“** (Runen erzählen in Bruchstücken von der sechsten Quelle) und **„Das verirrte Pilzkind“**; Gespräche im Dorf nach Kapitel 2 | Tests |
| M2.2.5 | Karten | Weg aus Tauwinkel in den Wald; `wald-1` bis `wald-3` und `wald-arena`: viel Vertikale, Baumhäuser, Hängebrücken, dunkle Höhlen unter Wurzeln, Abkürzungen, Rückkehr-Stellen für spätere Fähigkeiten; Ruck- und Hook-Stellen | Sichtprüfung + Durchlauf-Test |
| M2.2.6 | Quellfunke und Heranhooken | Sieg → Quellfunke → Tüftel baut **Heranhooken**; Übungsstelle in Tüftels Hof; Stellen in Kapitel 1, die erst mit Heranhooken gehen (Rückkehr lohnt sich) | Tests |
| M2.2.7 | Dorf nach Kapitel 2 | `quellen_befreit = 2` (mehr Farbe), Gespräche und Zurufe, Weltkarte; ggf. kleines Fest wie nach Kapitel 1 | Sichtprüfung |
| M2.2.8 | Musik und Sounds | Musik des Waldes und des Hüters (zum Anhören vorgelegt), Klänge für Schlange, Nüsse, Pilzwicht, Wurzeln, Kerne | Deine Hörprobe |
| M2.2.9 | Abnahme | Kapitel 2 durchspielen, speichern, fortsetzen | Deine Abnahme |

**Stand M2.2.1:** Neue Verhalten in der Simulation: `burrower` (Wurzelschlange: versteckt, wächst ab gut 6 Tiles Abstand in 0,7 s aus dem Boden, nur draußen verwundbar, gefährlich und hookbar), Wurf im Bogen für Schützen (`lob`, Eichhornpirat), Berührung mit `daze_ms` (Pilzwicht: bunter Rausch für 5,5 s, Elora läuft mit A-23 = × 0,55, schimmert in Regenbogenfarben, die Welt wabert leicht), `follower` (Pilzkind: folgt Elora, springt über Stufen, wartet an Lücken und Dornen, unverwundbar, harmlos). Arten in `creatures.toml` (`wurzelschlange`, `eichhornpirat`, `pilzwicht`, `pilzkind`), Grafiken aus den Entwürfen (E-312), dazu Plumm und das Pilzkind als Figuren. Ausprobieren: Training, F1 → „Gegner (Abenteuer)“.

**Stand M2.2.2:** Mit **Heranhooken** greift der Hook Sammelstücke (Bienen, Runen, Glitzersteine …) aus der Entfernung, Beute fliegt sofort zu Elora, und Hook-Schalter („Zugschalter“, im Editor unter den Auslösern des Schalters) reagieren nur noch mit dieser Fähigkeit – einmal je Hook-Schuss. Die Kerne des Wurzelwächters folgen mit dem Hüter (M2.2.3).

## Ablauf von Kapitel 2 (Vorschlag)

| # | Ort | Was passiert |
|---|---|---|
| 1 | Tauwinkel | Oma schickt Elora los; ein neuer Weg führt aus dem Dorf in den Wald |
| 2 | `wald-1` | Waldrand, erste Wurzelschlangen und Pilzwichte; erste Erinnerungsrune |
| 3 | `wald-2` | Baumhaus-Siedlung in den Kronen: **Uhu Plumm** sammelt Geschichten, das **Pilzkind** weint, es hat sich verirrt; Eichhornpiraten auf den Ästen |
| 4 | `wald-3` | Wurzelhöhlen, der Weg wird von Wurzeln versperrt; Quellstein vor der Arena; weitere Runen |
| 5 | `wald-arena` | Waldquelle, der **Wurzelwächter** schläft mitten auf ihr; Kampf |
| 6 | `wald-arena` | Sieg: Der Wächter war müde und verwirrt (wie die Hummel); Quellfunke; die Runen ergeben einen Satz über die **sechste Quelle** |
| 7 | Tauwinkel | Tüftel baut **Heranhooken**; Fest; Oma kündigt die Glutsandwüste an |
| 8 | frei | Pilzkind nach Hause bringen, alle Runen zu Plumm bringen, Rückkehr nach Kapitel 1 |

## Der Kampf gegen den Wurzelwächter (Vorschlag)

- **Phase 1:** Er steht in der Mitte, schlägt mit Wurzeln aus dem Boden (Warnung: Boden bebt, Erde bröckelt an der Stelle). Drei **Kerne** leuchten in seiner Rinde. Elora hookt einen Kern und zieht (Hook halten und weglaufen bzw. springen): Der Kern löst sich, der Wächter ist kurz offen.
- **Phase 2 – ab halbem Leben:** Er verschließt Teile der Arena mit Wurzelwänden; Elora muss über Hook-Stellen ausweichen.
- **Phase 3 – letzter Kern:** schneller, Wurzeln an zwei Stellen gleichzeitig.
- Treffer nur, solange ein Kern gelöst ist.

## Entscheidungen zu R2-M2.2

| # | Frage | Entscheidung |
|---|---|---|
| D-M22-01 | Weg in den Murmelwald | **Pfad am Westhang** von Tauwinkel hinauf (Stufen, Hook-Stellen), oben beginnt der Wald (E-306) |
| D-M22-02 | Kampf gegen den Wurzelwächter | **Wie vorgeschlagen** (E-307) |
| D-M22-03 | Pilzkind nach Hause bringen | **Es folgt Elora** (neue Technik Begleiter, wartet an schwierigen Stellen) (E-308) |
| D-M22-04 | Belohnung für die Runen | **Ausrüstungsstück und ein Tautropfen-Punkt**, dazu die Geschichte der sechsten Quelle (E-309) |
| D-M22-05 | Fest nach Kapitel 2 | **Wie nach Kapitel 1** (E-310) |
| D-M22-06 | Pilzwicht-Wirkung | **Bunter Rausch:** Elora schimmert ein paar Sekunden in Regenbogenfarben und läuft langsamer, die Welt wabert leicht (kindgerecht, ohne Bezug auf Drogen im Spiel) (E-311) |

## Technische Festlegungen (Vorschlag)

- Neue Verhalten und der Hüter in der Simulation (deterministisch), Werte als Daten in `creatures.toml`.
- **Zeitweise Wurzelwände** als Tiles, die die Simulation setzt und wieder entfernt (wie Bröckelboden, aber zurück zum Ursprung); Ereignis für Grafik und Klang.
- **Heranhooken von Gegenständen**: Beute und Sammelstücke als hookbare Ziele; Zugschalter als Abenteuer-Objekt.
- Karten aus einem Generator `editor/kapitel2.rs`; Durchlauf-Test um Kapitel 2 erweitert.
