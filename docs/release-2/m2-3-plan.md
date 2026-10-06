# R2-M2.3 – Kapitel 3: Glutsandwüste – Umsetzungsplan

Status: **Abgenommen** (E-327) · Entscheidungen E-315 bis E-327 · Grundlage: [`weltbuch.md`](weltbuch.md) §4.3 und §5, [`m2-2-plan.md`](m2-2-plan.md), E-243, E-296, E-314

## Ziel

Kapitel 3 ist von Anfang bis Ende spielbar: aus Tauwinkel in die Glutsandwüste, Karawanenführer **Sirup** mit seltenen Waren, eine **Oase**, die Wasser braucht, eine **verschüttete Ruine**, Spuren eines Wesens, das Farbe trinkt, der Hüter **Sandschlange** und die neue Fähigkeit **Stampfen**. Danach gibt Klonk den **Laser** (E-243), und Tauwinkel wird noch bunter (`quellen_befreit = 3`).

**Abnahme:** Kapitel 3 einmal durchspielen (etwa 60–90 Minuten mit Nebenaufgaben), dazwischen speichern und fortsetzen.

## Ausgangslage (aus R2-M1 bis R2-M2.2)

- Gegner-Verhalten: Läufer, Hüpfer, Schütze (auch im Bogen), Flieger, Hüter aus der Luft (`diver`), Wurzelschlange (`burrower`), Hüter am Boden (`warden`), Begleiter (`follower`); Berührung mit bunten Rausch.
- Fähigkeit **Stampfen** gibt es in der Simulation (A1.1): bricht Bröckelboden, Stoßwelle betäubt und schadet Gegnern.
- Bausteine: Karten-Generatoren (gespiegelt möglich), Ruck-Stelle, Zugtruhe, Hook-Blüten, Deko mit Zustand, Figuren mit `show_if`, Begleiter, Musik und Kampfmusik je Gebiet.
- Material **Sand** und das Release-Thema „Wüste“ (Himmel, Färbung) gibt es schon; Beschleuniger-Tiles (`<` `>`) als Laufbänder.
- Nach Kapitel 2 läuft „Spuren im Sand“ (Ankündigung, Schritt von Hand).

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| M2.3.0 ✅ | Entwürfe | Sirup und seine Karawane (Kamel o. ä.), Oasen-Hüterin (Figur der Nebenaufgabe), Sandkrabbe, Dünenwurm, Funkenmotte, Sandschlange (unter dem Sand, Auftauchen, Bogen durch die Luft, benommen, besiegt), Wasserschlauch, Ruinen-Steintafel, Quelle (verdorrt/befreit), Deko: Dünen, Felsbögen, Ruinen (Säulen, Tore, Treppen), Kakteen, Palmen, Oase, Zelte | Deine Auswahl |
| M2.3.1 ✅ | Neue Gegner | **Sandkrabbe**: gepanzert, Treffer von der Seite prallen ab, verwundbar von oben (Hammer von oben, Stampfen, Granate darüber). **Dünenwurm**: wandert unter dem Sand (Sandspur), springt in einem Bogen heraus und taucht wieder ein. **Funkenmotte**: fliegt, lässt Funken fallen, die kurz am Boden glühen | Tests + Sandbox |
| M2.3.2 ✅ | Treibsand und Hitze | neues Tile **Treibsand**: Elora sinkt langsam ein und läuft langsamer, Springen befreit, tief eingesunken kleiner Schaden und zurück an den Rand (E-318); **Hitzeflimmern** als Bildeffekt und **Hitze-Leiste** im HUD: Sonne füllt, Schatten (Zonen/Dächer) und Oase kühlen, voll = langsamer (E-320) | Tests |
| M2.3.3 ✅ | Hüter-Technik | **Sandschlange**: taucht unter dem Sand (nur Sandspur sichtbar), schießt an Eloras Stelle hoch, fliegt im Bogen und taucht wieder ein; verwundbar nur aufgetaucht; ab halbem Leben schneller, zum Schluss zwei Bögen hintereinander | Tests + Sandbox |
| M2.3.4 ✅ | Inhalte | Figuren Sirup und Oasen-Hüterin; Hauptaufgabe „Spuren im Sand“ bis zum Quellfunken; Nebenaufgaben **„Wasser für die Oase“** und **„Die verschüttete Ruine“**; Spuren des grauen Wanderers (Story); Gespräche im Dorf nach Kapitel 3; Klonk gibt den Laser | Tests |
| M2.3.5 ✅ | Karten | Weg aus Tauwinkel in die Wüste; `wueste-1` bis `wueste-3` und `wueste-arena`: Dünen, Treibsand-Laufbänder, Ruinen mit Bröckelböden (Rückkehr mit Stampfen), Oase, Karawanenlager, Stachelgruben; Ruck- und Zugstellen | Sichtprüfung + Durchlauf-Test |
| M2.3.6 ✅ | Quellfunke und Stampfen | Sieg → Quellfunke → Tüftel baut **Stampfen**; Übungsstelle in Tüftels Hof; Stellen in Kapitel 1 und 2, die erst mit Stampfen gehen | Tests |
| M2.3.7 ✅ | Dorf nach Kapitel 3 | `quellen_befreit = 3`, Fest, Gespräche und Zurufe, Weltkarte | Sichtprüfung |
| M2.3.8 ✅ | Musik und Sounds | Musik der Wüste und des Hüters (zum Anhören vorgelegt), Klänge für Krabbe, Wurm, Motte, Treibsand, Sandschlange | Deine Hörprobe |
| M2.3.9 ✅ | Abnahme | Kapitel 3 durchspielen, speichern, fortsetzen | Deine Abnahme |

**Stand M2.3.0–M2.3.7:** Entwürfe angenommen (E-321). Gegner Sandkrabbe (Panzer, E-317), Dünenwurm (`leaper`), Funkenmotte (Funken glühen am Boden). Treibsand-Tile `&` (E-318), Hitze-Leiste und Hitzeflimmern als Shader (E-320, E-321). Sandschlange (`serpent`, E-316). Inhalte nach [`m2-3-inhalte.md`](m2-3-inhalte.md) (E-322 bis E-325). Karten `wueste-1` bis `wueste-3`, `wueste-arena`, Hohlweg am Ostpfad (E-315); der Kessel der Arena liegt im Schatten. Stampfplatte in Tüftels Hof, Stampfkammern in `wiese-2`, `wald-1`, `wald-3`. Dorf nach Kapitel 3: `quellen_befreit = 3`, Fest, Lotte schenkt Kaktusfrüchte von Sirup, Klonk erklärt den Laser-Ausbau mit Glutstein, neue Zurufe von Pip, Lotte und Tüftel; Weltkarte: Glutquelle befreit (`spring = "glutquelle"`).

**Stand M2.3.8:** Musik (E-326): Glutsandwüste „Desert Loop“ (iamoneabe, CC0), Sandschlange „Hard Boss Battle 1“ (MintoDog, CC0). Klänge je Gegnerart: Sand spritzt (Wurm, Schlange), Sand bebt vor dem Sprung, zurück in den Sand (Kenney), Funken knistern, Panzer klackt (Kenney), die Schlange zischt, Schmatzen beim Hineingeraten in Treibsand (prozedural).

## Ablauf von Kapitel 3 (Vorschlag)

| # | Ort | Was passiert |
|---|---|---|
| 1 | Tauwinkel | Oma schickt Elora los; ein Weg führt aus dem Oberdorf über den Ostpfad hinaus in die Wüste |
| 2 | `wueste-1` | Dünenrand, erste Sandkrabben und Dünenwürmer, Treibsand-Laufbänder |
| 3 | `wueste-2` | Karawanenlager: **Sirup** mit seltenen Waren erzählt vom grauen Wanderer; nahebei die **Oase**, fast ausgetrocknet (Nebenaufgabe Wasser) |
| 4 | `wueste-3` | Ruinen eines alten Volkes: Tafeln mit Spuren eines Wesens, das Farbe trinkt; verschüttete Kammer (Nebenaufgabe); Funkenmotten; Quellstein |
| 5 | `wueste-arena` | Glutquelle in einem Sandkessel, die **Sandschlange** schläft darunter; Kampf |
| 6 | `wueste-arena` | Sieg: Die Schlange war verwirrt und müde; Quellfunke; graue Fußspuren führen Richtung Norden (Frostspitzen) |
| 7 | Tauwinkel | Tüftel baut **Stampfen**; Klonk gibt den **Laser**; Fest; Oma kündigt die Frostspitzen an |
| 8 | frei | Wasser zur Oase, Ruine freilegen (mit Stampfen), Rückkehr nach Kapitel 1 und 2 |

## Der Kampf gegen die Sandschlange (Vorschlag)

- **Phase 1:** Nur eine Sandspur wandert durch den Kessel. Unter Elora bebt der Sand (Warnung), dann schießt die Schlange hoch, fliegt in einem Bogen und taucht wieder ein. Solange sie in der Luft und kurz danach benommen am Boden liegt, ist sie verwundbar.
- **Phase 2 – ab halbem Leben:** schneller, Teile des Kessels werden zu Treibsand.
- **Phase 3 – letztes Viertel:** zwei Bögen hintereinander.
- Hammer und Granaten treffen; mit Stampfen (sobald vorhanden, z. B. in einer späteren Rückkehr) doppelter Schaden (E-316).

## Entscheidungen zu R2-M2.3

| # | Frage | Entscheidung |
|---|---|---|
| D-M23-01 | Weg in die Wüste | **Abzweig am Ostpfad:** hinter dem Ostpfad gabelt sich der Weg, nach Süden ein Hohlweg hinab in die Wüste (E-315) |
| D-M23-02 | Sandschlange und Stampfen | **Hammer und Granaten treffen** die aufgetauchte Schlange; Stampfen (nach dem Kampf) macht später doppelten Schaden (E-316) |
| D-M23-03 | Sandkrabbe | **Nur von oben verwundbar** (Schlag von oben, Stampfen, Granate darauf), von der Seite prallen Treffer ab (E-317) |
| D-M23-04 | Treibsand | **Sinkt langsam ein und verlangsamt, Springen befreit;** tief eingesunken kleiner Schaden und zurück an den Rand (E-318) |
| D-M23-05 | Oasen-Aufgabe | **Wasserschlauch:** an einer Quelle in den Ruinen füllen und an drei verdorrten Stellen der Oase gießen, jede blüht auf (E-319) |
| D-M23-06 | Hitze | **Hitzeflimmern als Bildeffekt und eine Hitze-Leiste:** in der prallen Sonne füllt sie sich, im Schatten und an der Oase kühlt sie ab; voll = Elora wird langsamer (E-320) |

## Technische Festlegungen (Vorschlag)

- Neue Verhalten und der Hüter in der Simulation (deterministisch), Werte als Daten in `creatures.toml`.
- **Treibsand** als neues Tile in Kollision, Karte und Editor (wie Eis und Beschleuniger), Grafik als Material.
- Panzer der Sandkrabbe: Treffer bewerten die Richtung (Schlag von oben, Stoßwelle), sonst Abprallen wie beim Hüter.
- Karten aus einem Generator `editor/kapitel3.rs`; Durchlauf-Test um Kapitel 3 erweitert.
