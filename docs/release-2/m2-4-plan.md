# R2-M2.4 – Kapitel 4: Frostspitzen – Umsetzungsplan

Status: **Entwurf zur Freigabe** · Entscheidungen E-340 bis E-343 · Grundlage: [`weltbuch.md`](weltbuch.md) §4.4 und §5, [`m2-3-plan.md`](m2-3-plan.md), [`w1-plan.md`](w1-plan.md), E-228, E-243, E-320

## Ziel

Kapitel 4 ist von Anfang bis Ende spielbar: aus Tauwinkel hinauf in die Frostspitzen, Bergführerin
**Flocke** im verlassenen Bergdorf, die neue Fähigkeit **Eisgriff** schon mitten im Kapitel,
**Kälte**, die Elora an Feuerstellen vertreibt, Eiszapfen, Lawinen und dünnes Eis, verlorene
Kletterer und Eiskristalle für Klonk, die Hüterin **Eiskönigin Kristella** und ihre Warnung:
„Er ist einsam, nicht böse.“ Danach wird Tauwinkel noch bunter (`quellen_befreit = 4`), und Oma
kündigt die Sternschlucht an.

**Abnahme:** Kapitel 4 einmal durchspielen (etwa 60–90 Minuten mit Nebenaufgaben), dazwischen
speichern und fortsetzen.

## Grundsatzentscheidungen

| # | Frage | Entscheidung |
|---|---|---|
| E-340 | Eisgriff | **Mitten im Kapitel:** Flocke gibt Elora im Bergdorf Steigkrallen (= Eisgriff); ab da öffnen sich Wege, der Hüterkampf nutzt die Wände. Der Quellfunke nach dem Sieg **stärkt** den Eisgriff |
| E-341 | Kampf gegen Kristella | **Frostwellen und Wände:** Sie schwebt über einer Eishalle und friert den Boden in Wellen ein (frisch gefrorener Boden schadet); Elora rettet sich an Kletterwände; nach dem Frosthauch ist Kristella erschöpft, sinkt herab und ist verwundbar; später fallen Eiszapfen |
| E-342 | Kälte | **Kälte-Leiste** (Gegenstück zur Hitze, E-320): draußen füllt sie sich, im Schneesturm schneller; an Feuerstellen und unter Dächern wärmt Elora sich auf; voll = Elora wird langsamer |
| E-343 | Neues Gelände | **Eiszapfen, Lawinen, dünnes Eis** (keine Sprungfelder) |

## Ausgangslage (aus R2-M1 bis R2-W1)

- **Eisgriff** gibt es in der Simulation (A1.1, E-228): in der Luft gegen ein **Kletterwand-Tile**
  laufen, kurz haften (A-06: 1,0 s), abspringen; im Fähigkeitenbaum „Fester Griff“ (Haftdauer).
- Tiles **Eis** (rutschig), **Kletterwand**, **Bröckelboden** (bricht beim Stampfen), Treibsand.
- **Wetter** (R2-W1): Schnee und Schneesturm mit Wind, der Elora in der Luft schiebt, nasser
  Boden; Gebiet `frostspitzen` in `worldmap.toml` (Karten `frost-`, trüb: Schneesturm, sonst
  Schnee oder schön).
- **Hitze-Leiste** (E-320) mit Zonen `schatten…`/`oase…` und Dach-Erkennung – Vorlage für die Kälte.
- Gegner-Bausteine: Läufer, Hüpfer, Schütze (auch im Bogen), Flieger (mit Abwurf), `diver`,
  `burrower`, `leaper`, `warden`, `serpent`; Hüter-Technik mit Phasen, Warnungen, Erschöpfung.
- Material **Eiskristall** und Ausbauten bei Klonk, die es brauchen (`upgrades.toml`); Aufgabe
  „Der Ruf der Frostspitzen“ (Ankündigung, Schritt von Hand).

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| M2.4.0 | Entwürfe | Flocke, Kletterer (3), Kristella (schwebend, Frosthauch, erschöpft, beruhigt), Schneeballrobbe, Eisspitzen-Fledermaus, Frostgeist, Steigkrallen, Eiszapfen, Lawinen-Schneeball, dünnes Eis und Eiswasser, Feuerstelle, Quelle (vereist/befreit); Deko: Gipfel, Tannen im Schnee, Berghütten, Seilbrücken, Gletscher | Deine Auswahl |
| M2.4.1 | Gelände in der Simulation | **Eiszapfen** (zittern, fallen, zerschellen), **dünnes Eis** (neues Tile: bricht nach kurzem Stehen oder sofort beim Stampfen, wächst nach), **Eiswasser** (neues Tile: kleiner Schaden, zurück an den Rand), **Lawinen** (Zone: Stampfen oder Explosion löst rollende Schneebrocken aus) | Tests |
| M2.4.2 | Kälte | Kälte-Leiste im HUD, Frostrand als Bildeffekt (Post-Shader); Feuerstellen und Hütten wärmen; Werte als Tuning | Tests |
| M2.4.3 | Neue Gegner | **Schneeballrobbe**: rutscht auf dem Bauch heran, hält an und wirft Schneebälle im Bogen. **Eisspitzen-Fledermaus**: hängt schlafend an der Decke, stürzt herab, wenn Elora darunter ist, flattert zurück. **Frostgeist**: schwebt durch Wände, Berührung lässt Elora kurz erstarren | Tests + Sandbox |
| M2.4.4 | Hüter-Technik | **Kristella** nach E-341, Phasen siehe unten | Tests + Sandbox |
| M2.4.5 | Inhalte | Figuren Flocke und Kletterer; Hauptaufgabe bis zum Quellfunken mit Steigkrallen in der Mitte; Nebenaufgaben **„Verlorene Kletterer“** und **„Eiskristalle für Klonk“**; Spuren des Dürren; Gespräche im Dorf nach Kapitel 4 | Tests |
| M2.4.6 | Karten | Bergsteig aus Tauwinkel; `frost-1` bis `frost-3` und `frost-arena` | Sichtprüfung + Durchlauf-Test |
| M2.4.7 | Quellfunke und Dorf | Sieg → Quellfunke → Tüftel stärkt den Eisgriff; Kletterstellen in Kapitel 1–3 (Rückkehr lohnt); `quellen_befreit = 4`, Fest, Weltkarte; Oma kündigt die Sternschlucht an | Tests + Sichtprüfung |
| M2.4.8 | Musik und Sounds | Musik der Frostspitzen und Kristellas (zum Anhören vorgelegt), Klänge für Robbe, Fledermaus, Geist, Eiszapfen, Lawine, brechendes Eis, Feuerstelle, Kristella | Deine Hörprobe |
| M2.4.9 | Abnahme | Kapitel 4 durchspielen, speichern, fortsetzen | Deine Abnahme |

## Ablauf von Kapitel 4 (Vorschlag)

| # | Ort | Was passiert |
|---|---|---|
| 1 | Tauwinkel | Oma schickt Elora los; oben im Oberdorf führt ein **Bergsteig** nach Norden, bisher von einem Eisblock versperrt – mit **Stampfen** zerbricht er |
| 2 | `frost-1` Gletscherfuß | Erste Hänge und Eisflächen, Schneeballrobben, Eiszapfen in einem Felsgang, dünnes Eis über einem Eiswasser-Becken; Kälte-Leiste mit der ersten Feuerstelle eingeführt |
| 3 | `frost-2` Verlassenes Bergdorf | **Flocke** wartet in einer Hütte; Elora holt ihr Seil aus einem vereisten Keller zurück (Fledermäuse, dünnes Eis) und bekommt die **Steigkrallen (Eisgriff)**; gleich danach ein Kamin aus Kletterwänden hinaus. Erster Kletterer, Klonks Auftrag (Eiskristalle) |
| 4 | `frost-3` Gipfelgrat | Schneesturm-Abschnitte mit Wind, Lawinenhänge, Kletterwand-Schächte, Frostgeister; graue Spuren und eine vereiste Stelle, an der jemand „Farbe getrunken“ hat; Quellstein vor der Halle |
| 5 | `frost-arena` Eishalle | Frostquelle unter einer Eisdecke, Kristella schwebt darüber; Kampf |
| 6 | `frost-arena` | Sieg: Kristella beruhigt sich – „Er ist einsam, nicht böse.“ Quellfunke; sie zeigt nach Osten, zur Sternschlucht |
| 7 | Tauwinkel | Tüftel stärkt den Eisgriff mit dem Quellfunken; Fest; Oma kündigt die Sternschlucht an |
| 8 | frei | Kletterer retten, Eiskristalle für Klonk, Kletterstellen in Kapitel 1–3 |

## Der Kampf gegen Kristella (Vorschlag nach E-341)

- **Arena:** Eishalle, etwa zwei Bildschirme breit; links und rechts sowie an zwei Säulen
  Kletterwände; Boden aus Eis; die Kälte-Leiste ruht im Kampf.
- **Phase 1:** Kristella schwebt oben und zieht von einer Seite eine **Frostwelle** über den Boden:
  Reif kriecht als Warnung voraus, dann friert der Boden – wer darauf steht, nimmt Schaden. Elora
  springt an eine Kletterwand und hält sich (Eisgriff) oder springt über die Welle. Nach drei
  Wellen ist Kristella **erschöpft**, sinkt herab und ist kurz verwundbar (Hammer, Granaten, Laser).
- **Phase 2 – ab halbem Leben:** Wellen von beiden Seiten nacheinander; zwischen den Wellen fallen
  **Eiszapfen** von der Decke (mit Zittern als Warnung).
- **Phase 3 – letztes Viertel:** Schneesturm in der Halle (Wind schiebt Elora von der Wand),
  kürzere Pausen, eine Welle mehr.

## Offen zur Freigabe

| # | Frage | Vorschlag |
|---|---|---|
| D-M24-01 | Weg in die Frostspitzen | **Bergsteig oben im Oberdorf** nach Norden, versperrt von einem Eisblock, den Elora mit Stampfen zerbricht (nutzt die Fähigkeit aus Kapitel 3) |
| D-M24-02 | Steigkrallen | Flocke gibt sie, nachdem Elora ihr **Seil aus dem vereisten Keller** geholt hat; der Keller kommt noch ohne Eisgriff aus |
| D-M24-03 | Quellfunke | **Eisgriff gestärkt:** doppelte Haftdauer, und Elora kann an der Wand ein Stück **hinaufziehen** (Taste hoch) statt nur abzurutschen |
| D-M24-04 | Kälte-Leiste | Füllt sich draußen in etwa 60 s (Schneesturm etwa 30 s), Feuerstelle wärmt in etwa 3 s ganz auf, Dach in etwa 10 s; voll = Tempo × 0,7 wie die Hitze; Ausrüstung mit Kälteschutz (Mütze, Umhang) verlangsamt das Füllen. In Arenen ruht sie |
| D-M24-05 | Eiswasser | Kleiner Schaden (1) und zurück an den Rand, wie ganz eingesunkener Treibsand (E-318); dünnes Eis wächst nach etwa 4 s nach |
| D-M24-06 | Lawinen | Nur in markierten Zonen; Auslöser Stampfen, Granate oder ein Schritt auf eine Auslöse-Stelle; 6–8 rollende Schneebrocken (2 Schaden, Stoß), Schutz in Nischen oder an einer Kletterwand darüber |
| D-M24-07 | Frostgeist | Schwebt langsam durch Wände auf Elora zu; Berührung lässt sie 0,6 s erstarren (wie Rausch, aber starr); verwundbar durch alles, außerhalb der Felsen sichtbar heller |
| D-M24-08 | Kletterer | **Drei Kletterer**, je einer in `frost-1` bis `frost-3`, nur mit Eisgriff erreichbar (der erste nach der Rückkehr); Belohnung von Flocke: Kälteschutz-Mütze |
| D-M24-09 | Eiskristalle | **Acht Kristalle** versteckt (unter dünnem Eis, hinter Lawinenhängen, in Kletterschächten); Klonk baut daraus einen Ausbau nach Wahl |
| D-M24-10 | Wetter | Gletscherfuß und Bergdorf: Schnee (trüb: Schneesturm); Gipfelgrat: Schneesturm-Abschnitte fest in der Karte; Eishalle: schön (D-W1-01) |

## Technische Festlegungen (Vorschlag)

- Neue Tiles **dünnes Eis** und **Eiswasser** in Kollision, Kartenformat und Editor (wie Treibsand).
- **Eiszapfen** als Karten-Objekt mit Zustand (hängt, zittert, fällt, zerschellt) in der
  Simulation; **Lawinen** als Zonen `lawine…` mit Auslöser und rollenden Körpern (deterministisch).
- **Kälte** teilt sich die Technik der Hitze: eine Temperatur-Leiste je Gebiet (`hot` / neu
  `cold` in `worldmap.toml`), Zonen `feuer…` wärmen, Dach-Erkennung wie beim Schatten.
- Gegner und Kristella als neue Verhalten in `elora-sim` (deterministisch), Werte in
  `creatures.toml`; Frostgeist ignoriert die Kollision.
- Karten aus einem Generator `editor/kapitel4.rs`; Durchlauf-Test um Kapitel 4 erweitert.
- Musik und Klänge aus freien Quellen (CC0), zum Anhören vorgelegt (E-295).
