# Fortschrittssystem des Abenteuers (O-204)

Status: **entschieden** (E-241 bis E-245, A1.3) · Rahmen: E-206, E-212, E-214/E-215, E-219, E-220, E-239, [Weltbuch §6](world-book.md)

Grundsatz: Hook, Sprung und Doppelsprung bleiben, wie sie sind (E-212). **Fortschritt verändert nie Lauftempo, Sprunghöhe oder Hook-Zug**, sondern Leben, Schaden, die fünf Gebietsfähigkeiten, Beute und Komfort.

Alle Zahlen sind Startwerte (Kürzel **P-xx**) und stehen später in Datendateien unter `assets/adventure/`.

## 1. Stufen und Erfahrung

| # | Wert | Vorschlag |
|---|---|---|
| P-01 | Höchststufe | **30** (Ende der Hauptgeschichte etwa Stufe 20–22, danach Nebeninhalte und Quellen-Prüfungen) |
| P-02 | Erfahrung bis zur nächsten Stufe | **15 + 10 × Stufe** (Stufe 1→2: 25, 10→11: 115, 29→30: 305; bis Stufe 30 insgesamt 4785) |
| P-03 | Erfahrung aus Gegnern | aus `creatures.toml` (heute 5–8), Bosse 100–300 |
| P-04 | Erfahrung aus Aufgaben | Nebenaufgaben 30–120, Hauptaufgaben 80–250 |
| P-05 | Je Stufe | **1 Tautropfen-Punkt**; alle **2 Stufen +1 Leben** (10 → 24 auf Stufe 30, E-241) |
| P-06 | Stufenaufstieg | füllt Leben auf, kurze Anzeige mit Effekt |

## 2. Fähigkeitenbaum (bei Tüftel)

Drei Zweige, jeder Knoten hat 1–3 Ränge zu je 1 Punkt. Ein Knoten wird frei, wenn der darüber mindestens Rang 1 hat. **Insgesamt 35 Ränge** (im Vorschlag stand irrtümlich 41); bis Stufe 30 gibt es 29 Punkte plus 2 aus besonderen Aufgaben, also 31 – nicht alles (E-242): Die Wahl zählt. Bewegungs-Knoten brauchen die jeweilige Gebietsfähigkeit.

| Zweig | Knoten (Ränge) | Wirkung je Rang |
|---|---|---|
| **Bewegung** | Schneller Ruck (3) | Abklingzeit Hook-Ruck −150 ms |
| | Weiter Hook (2) | Hook-Länge +5 % |
| | Starkes Stampfen (3) | Stoßwelle +16 Radius, +1 Schaden |
| | Fester Griff (2) | Haftdauer +0,4 s |
| | Weites Gleiten (2) | Fallen beim Gleiten −0,4 |
| **Kampf** | Kraft (3) | Schaden aller Waffen +10 % |
| | Schnelle Hand (3) | Feuerverzögerung −8 % |
| | Wucht (2) | Rückstoß auf Gegner +25 % |
| | Munitionstasche (2) | Munition +2 |
| | Betäubender Hammer (1) | Hammer betäubt Gegner 0,5 s |
| **Quelle** | Mehr Leben (3) | +1 Leben |
| | Tropfen-Magnet (2) | Beute-Magnet +48 |
| | Glanz-Fund (2) | +10 % Glanztropfen |
| | Langer Schutz (2) | Schutz nach Treffer +250 ms |
| | Heilblumen (2) | Heilpflanzen heilen +1 |
| | Zweite Chance (1) | einmal je Karte mit 3 Leben weitermachen statt zu sterben |

## 3. Waffen und Ausbau (bei Klonk)

| # | Thema | Vorschlag |
|---|---|---|
| P-10 | Waffen im Abenteuer | Start mit **Hammer**; **Granatwerfer** am Ende von Kapitel 1 (Belohnung von Klonk), **Laser** am Ende von Kapitel 3 |
| P-11 | Munition | **wie im Mehrspieler über Pickups und Truhen** (E-243) |
| P-12 | Ausbaustufen | je Waffe **3 Stufen** (I–III) |
| P-13 | Kosten | Glanztropfen + Material des passenden Gebiets, z. B. Hammer I: 40 + 3 Bernstein, II: 120 + 4 Harz, III: 300 + 3 Sternsplitter |

| Waffe | Stufe I | Stufe II | Stufe III |
|---|---|---|---|
| Hammer | Wucht: +1 Schaden | Reichweite +30 % | Schockwelle: trifft alles um Elora |
| Granatwerfer | Explosion +20 % | Splitter: 3 kleine Nach-Explosionen | +2 Munition |
| Laser | +1 Abpraller | Durchschlag: trifft bis zu 3 Gegner | Ladezeit −25 % |

Materialien je Gebiet: Bernstein (Blütenwiesen), Harz (Murmelwald), Glutstein (Glutsandwüste), Eiskristall (Frostspitzen), Sternsplitter (Sternschlucht).

## 4. Ausrüstung und Inventar

| # | Thema | Vorschlag |
|---|---|---|
| P-20 | Plätze | Hut, Umhang, Stiefel, Anhänger (E-206) |
| P-21 | Boni | 1–2 kleine Boni je Stück aus: Leben, Rüstung, Schaden %, Beute-Magnet, Glanztropfen %, Schutzzeit, Fähigkeitswerte (wie im Baum). **Keine Tempo- oder Sprungboni** |
| P-22 | Seltenheit | gewöhnlich (1 Bonus), selten (2 Boni), Hüter-Fund (2 Boni + Besonderheit) |
| P-23 | Rüstung | nur aus Ausrüstung (E-239); wird an Speicherpunkten aufgefüllt |
| P-24 | Inventar | **ohne Platzgrenze**; Reiter Ausrüstung, Verbrauchsgegenstände, Materialien, Wichtiges (Schlüssel, Aufgabengegenstände – nicht verkaufbar) |
| P-25 | Verbrauch | Heiltrank (+5 Leben), Tautrank (Hook-Ruck ohne Abklingzeit, 10 s); höchstens 5 je Art |
| P-26 | Verkaufen | bei Lotte für 40 % des Kaufpreises |

## 5. Tod und Speichern

| # | Thema | Vorschlag |
|---|---|---|
| P-30 | Verlust beim Tod (E-220) | **25 %** der seit dem letzten Speicherpunkt gesammelten Glanztropfen (abgerundet); alles andere bleibt |
| P-31 | Speicherpunkte | Quellstein: speichert, füllt Leben und Rüstung auf |
| P-32 | Spielstand | 3 Plätze (E-219) im Benutzerverzeichnis (`saves/platz-1.esav` …), **gepackt mit Prüfsumme, nicht lesbar** (E-245), mit Formatversion; ein defekter Platz wird gemeldet, nie überschrieben |
| P-33 | Inhalt | Stufe, Erfahrung, Leben, Glanztropfen, Baum, Inventar, Ausrüstung, Waffenstufen, Fähigkeiten, Aufgaben, Weltzustand (Schalter, Türen, Bröckelboden, Truhen, besiegte Bosse, Folgen aus Gesprächen), Ort (Karte + Speicherpunkt), Spielzeit |
