# R2-M2.4 – Inhalte von Kapitel 4 (Entwurf, M2.4.5)

Status: **Freigegeben und eingebaut** (Texte freigegeben 2026-10-07, E-346) · Grundlage: [`m2-4-plan.md`](m2-4-plan.md), [`world-book.md`](world-book.md) §4.4 und §5, E-340 bis E-345, D-M24-01 bis D-M24-10

Deutsche Texte; die englischen kommen beim Einbau dazu. Bedingungen und Folgen in der Schreibweise
der Inhaltsdateien ([`../handbook/adventure-content.md`](../handbook/adventure-content.md)).

## 1. Figuren

| Id | Name | Wo | Stimme | Wesen |
|---|---|---|---|---|
| `flocke` | Bergführerin Flocke | Hütte im Bergdorf (`frost-2`) | klar, kräftig (1,05) | tatkräftig, ein bisschen stur, lacht laut; will seit Jahren auf den Gipfel |
| `bolle` | Kletterer Bolle | Felsnische in `frost-1` (nur mit Eisgriff) | tief (0,9) | gemütlich, hat sich „nur kurz hingesetzt“ |
| `kiesel` | Kletterin Kiesel | Gletscherspalte in `frost-2` | hell (1,2) | flink, ungeduldig, schämt sich ein wenig |
| `wicke` | Kletterer Wicke | Sims am Gipfelgrat (`frost-3`) | zittrig (1,1) | ängstlich, zählt Schneeflocken gegen die Angst |
| `kristella` | Eiskönigin Kristella | Eishalle, nach dem Kampf (`show_if = "merker besiegt.kristella"`) | ruhig, kühl (0,8) | streng und stolz, nach dem Kampf sanft und nachdenklich |
| `graue-stelle` | Graue Stelle | Gipfelgrat (`frost-3`) | stumm, fest | Eis ohne Farbe, wo der Dürre getrunken hat |

Die Kletterer verschwinden aus ihrer Nische, sobald Elora mit ihnen gesprochen hat
(`show_if = "nicht merker kletterer.<id>"`), und sitzen danach in Flockes Hütte.

## 2. Hauptaufgabe „Der Ruf der Frostspitzen“ (`frostspitzen`, ersetzt den Schritt „bald“)

> Die vierte Quelle liegt in den Frostspitzen im Norden. Dorthin führten die grauen Spuren.

| Schritt | Text | Ziel |
|---|---|---|
| `aufbruch` | Oben im Oberdorf den Bergsteig freilegen | `frost-1` erreichen |
| `flocke` | Im verlassenen Bergdorf nach jemandem suchen | mit Flocke sprechen |
| `seil` | Flockes Seil aus dem vereisten Keller holen | Seil zu Flocke bringen |
| `grat` | Mit den Steigkrallen hinauf zum Gipfelgrat | `frost-3` erreichen |
| `quelle` | Die Eishalle der Frostquelle finden | `frost-arena` erreichen |
| `hueter` | Kristella beruhigen | Kristella besiegen |
| `funke` | Tüftel den Quellfunken bringen | Quellfunke zu Tüftel |
| `fest` | Mit Oma Pfütze am Brunnen feiern | mit Oma sprechen |

Lohn: 400 Erfahrung, 180 Glanztropfen. Danach beginnt die Ankündigung **„Das Lied der Sterne“**
(`sternschlucht`, Schritt von Hand, „bald“).

## 3. Nebenaufgaben

### „Verlorene Kletterer“ (`kletterer`, von Flocke, D-M24-08)

| Schritt | Text | Ziel |
|---|---|---|
| `finden` | Die drei verschollenen Kletterer finden | Merker `kletterer.gefunden` = 3 |
| `bericht` | Flocke Bescheid geben | mit Flocke sprechen |

Jeder Kletterer zählt `kletterer.gefunden` hoch und macht sich auf den Weg zur Hütte. Bolle in `frost-1`
sitzt in einer Nische, die nur mit dem Eisgriff erreichbar ist (also erst auf dem Rückweg). Lohn:
100 Erfahrung, 1 Tautropfen-Punkt, **Bommelmütze**.

### „Klarkristalle für Klonk“ (`klarkristalle`, von Klonk, D-M24-09)

| Schritt | Text | Ziel |
|---|---|---|
| `sammeln` | Acht Klarkristalle in den Frostspitzen finden | 8 Klarkristalle haben |
| `bringen` | Klonk die Kristalle bringen | 8 Klarkristalle zu Klonk |

Die Kristalle liegen versteckt: unter dünnem Eis, hinter Lawinenhängen, oben in Kletterschächten.
Klonk schleift daraus **einen Anhänger nach Wahl** (siehe Gespräch). Lohn zusätzlich: 100 Erfahrung.

## 4. Gegenstände und Flockes Vorrat

| Id | Art | Name | Wirkung | Preis |
|---|---|---|---|---|
| `seil` | Schlüssel | Flockes Seil | „Dick, rau und mit drei Knoten, die Flocke ‚Glück, Mut und Abendbrot‘ nennt.“ | – |
| `klarkristall` | Sammelstück | Klarkristall | „Ganz klar. Wenn man hindurchsieht, wirkt die Welt ein bisschen ruhiger.“ | – |
| `kraeutertee` | Verbrauch | Kräutertee | heilt 2 und **leert die Kälte-Leiste** (neu: Wirkung `warm`) | 12 |
| `bommelmuetze` | Hut | Bommelmütze | **Kälte-Leiste füllt sich 40 % langsamer** | – (Lohn) |
| `fellstiefel` | Stiefel | Fellstiefel | Kälte-Leiste −20 %, Rüstung +1 | 220 |
| `wuchtkristall` | Anhänger | Wuchtkristall | Hammer +1 Schaden | – (Klonk) |
| `sprengkristall` | Anhänger | Sprengkristall | Explosion +20 % | – (Klonk) |
| `lichtkristall` | Anhänger | Lichtkristall | Laser +1 Abpraller | – (Klonk) |

Flockes Vorrat (Laden `flocke`): Kräutertee, Eiskristall (25), Fellstiefel; Rabatt ab Zuneigung 5 (10 %).
Gegner der Frostspitzen lassen Eiskristall fallen (schon so), Kristella 3 Eiskristalle.

## 5. Gespräche

### Flocke

**Begrüßung** (erster Besuch):
> Ha! Endlich mal jemand, der nicht vor dem bisschen Schnee wegläuft! Ich bin Flocke, Bergführerin. Na ja – Bergführerin ohne Berg, seit der Gipfel zugefroren ist. Und ohne Seil. Das ist das eigentliche Problem.

- *(neugierig)* „Was ist mit deinem Seil?“ → **Seil**
- *(freundlich)* „Warum ist das Dorf so leer?“ → „Die Leute sind ins Tal gezogen, als es kälter und kälter wurde. Kälter als normal, meine ich. Seit da oben etwas Graues herumstapft, friert sogar das Feuer.“ → **Seil**
- *(frech)* „Bergführerin ohne Berg? Und ohne Seil?“ → „HA! Frech! Gefällt mir.“ (Zuneigung +1) → **Seil**

**Seil** (startet Schritt `seil`):
> Mein Seil liegt im Keller unter der alten Käserei. Ich wollte es holen, aber da unten hängen Fledermäuse mit Eiszapfen an den Flügeln. Ich mag Fledermäuse. Nur nicht, wenn sie mir ins Gesicht fliegen. Holst du es mir? Der Boden dort ist dünnes Eis – nicht trödeln!

- „Ich hole es.“ → `quest frostspitzen weiter`
- „Und dann?“ → „Dann gehen wir auf den Gipfel! Also, du. Ich halte das Seil.“ → `quest frostspitzen weiter`

**Erinnerung:** „Der Keller unter der Käserei. Dünnes Eis! Nicht stehen bleiben!“

**Seil zurück** (Schritt `seil`, mit Seil):
> Mein Seil! Glück, Mut und Abendbrot – alle drei Knoten noch dran! Hier, nimm dafür meine Steigkrallen. Damit hältst du dich an den Kletterwänden fest – an den rauen, gestreiften Felsen. Spring dagegen, halt dich fest, und dann: abspringen! Ganz einfach. Na ja. Fast.

→ `nimm seil 1`, `faehigkeit eisgriff`, `quest frostspitzen weiter`

- *(freundlich)* „Danke, Flocke!“ → **Kletterer**
- „Wohin jetzt?“ → „Hoch! Durch den Kamin hinter meiner Hütte. Oben am Grat wird's stürmisch.“ → **Kletterer**

**Kletterer** (startet die Nebenaufgabe):
> Ach, und … drei meiner Kletterer sind nicht zurückgekommen. Bolle, Kiesel und Wicke. Die drei sind gut, wirklich! Aber der Sturm … Wenn du sie siehst, schick sie heim. Ich mache Tee.

- *(freundlich)* „Ich halte die Augen offen.“ → `quest kletterer start`
- „Vielleicht später.“

**Bericht** (alle drei gefunden):
> Alle drei sitzen an meinem Ofen und streiten, wer am längsten durchgehalten hat. Danke, kleiner Tropfen. Hier – meine alte Bommelmütze. Damit friert dir nicht mal die Laune ein.

**Zurufe:** „Tee ist fertig!“ · „Kälte ist nur Wärme, die gerade woanders ist.“ · „Am Feuer wird's wieder warm!“

### Kletterer
- **Bolle:** „Oh, hallo! Ich hab mich nur kurz hingesetzt. Vor drei Tagen. Gemütlich hier. Na gut, ich geh heim – Flocke macht bestimmt Tee.“ → `merker kletterer.bolle = 1`, `merker kletterer.gefunden +1`
- **Kiesel:** „Sag Flocke nichts! Ich bin nicht reingefallen, ich … erkunde die Spalte. Gründlich. Ja, ich komme ja schon mit raus.“ → `merker kletterer.kiesel = 1`, `merker kletterer.gefunden +1`
- **Wicke:** „Vierhundertzwölf … vierhundertdreizehn … oh! Ein Tropfen! Ich zähle Schneeflocken, damit ich keine Angst habe. Du hast mich verzählen lassen. Danke. Ich geh lieber runter.“ → `merker kletterer.wicke = 1`, `merker kletterer.gefunden +1`
- **In der Hütte** (danach): Bolle „Bester Platz am Ofen.“ · Kiesel „Ich war NICHT verloren.“ · Wicke „Hier drin schneit es nicht. Ich hab nachgezählt.“

### Graue Stelle (fest, `frost-3`)
> Das Eis hier ist grau und stumpf, als hätte jemand alles Blau herausgetrunken. Daneben ein Abdruck im Schnee – ein großer, müder Fuß.

### Kristella (nach dem Kampf)
> Genug. Du bist zäher als du aussiehst, kleiner Tropfen. Ich … war nicht ich selbst. Der Graue kam in einer klaren Nacht und trank von meiner Quelle. Danach hörte ich nur noch Sturm in meinem Kopf.
>
> Doch ich habe ihn gesehen, als er ging. Er weinte. Er ist einsam, nicht böse. Er sucht etwas, das ihm jemand genommen hat – und er weiß nicht mehr, wer.
>
> Er zog nach Osten, zur Sternschlucht. Nimm den Funken. Und wenn du ihn findest … hör ihm zu.

### Tüftel (Schritt `funke`, D-M24-03)
> Blau! Eisblau! Und so kühl, dass meine Brille beschlägt. Gib mal deine Steigkrallen … *klirr* … *zisch* … So! Jetzt halten sie doppelt so lange, und wenn du an der Wand nach oben drückst, ziehst du dich sogar ein Stück hinauf. Kletterwände überall im Tauland – jetzt gehören sie dir!

→ `merker eisgriff.stark = 1`, `nimm quellfunke 1`, `quest frostspitzen weiter`

### Klonk
- **Auftrag** (Kapitel 4 läuft): „Eiskristalle schmelzen nicht. Weißt du, was nicht mal *die* können? So klar sein wie Klarkristalle. Acht Stück, irgendwo in den Bergen. Bring sie mir, und ich schleif dir was Hübsches. Hübsch und nützlich. Hübsch ist mir egal.“ → `quest klarkristalle start`
- **Erinnerung:** „Acht. Nicht sieben. Ich zähle nach.“
- **Bringen:**
  > Acht. Ich hab nachgezählt. Zweimal. Was soll's werden?
  - „Etwas für den Hammer.“ → `gib wuchtkristall 1`
  - „Etwas für die Granaten.“ → `gib sprengkristall 1`
  - „Etwas für den Laser.“ → `gib lichtkristall 1`
  
  danach: „Trag ihn mit Würde. Oder wenigstens ohne ihn zu verlieren.“ → `nimm klarkristall 8`, `quest klarkristalle fertig`

### Oma Pfütze
- **Auftrag** (nach dem Fest von Kapitel 3): „Die Frostquelle, Kind. Sie hat mich als junges Ding so klar denken lassen, dass ich einmal sogar die Steuern verstanden habe. Oben im Oberdorf liegt ein Eisblock vor dem alten Bergsteig. Du weißt ja jetzt, wie man aufstampft. Und zieh dich warm an!“
- **Unterwegs:** „Hast du auch eine Mütze? Ohne Mütze wird man zum Eiszapfen.“
- **Fest:** „Blau wie ein Wintermorgen! Und klar – endlich klar in meinem Kopf. Kristella sagt, er ist einsam? … Einsam. Wie ein Brunnen ohne Lied. Ach, warum fällt mir das jetzt ein? Geh nach Osten, Kind, in die Sternschlucht. Dort wohnt Luma, die Sternenweberin. Sie kennt alle Lieder.“ → `quest frostspitzen fertig`, `quest sternschlucht start`, Fest

### Schilder
- `schild-bergsteig` (Oberdorf): „↑ Frostspitzen – Bergsteig. Warm anziehen!“
- `schild-kaelte` (`frost-1`): „Kälte macht steif. Am Feuer und unter Dächern wärmst du dich auf.“
- `schild-eis` (`frost-1`): „Dünnes Eis! Nicht stehen bleiben. Und NICHT aufstampfen.“
- `schild-lawine` (`frost-3`): „Lawinengefahr! Lärm und Stampfen vermeiden.“
- `schild-kamin` (`frost-2`, hinter der Hütte): „Kletterkamin. Nur mit Steigkrallen.“

## 6. Spuren des Dürren

Graue Stellen im Eis (Deko `grauspur-eis`) in `frost-1` bis `frost-3`, dazu die Figur `graue-stelle` am
Gipfelgrat; vom Grat führen graue Fußspuren nach Osten aus der Arena hinaus. Kein Auftritt des Dürren.

## 7. Dorf nach Kapitel 4 (Ausblick auf M2.4.7)

`quellen_befreit = 4`, Fest; neue Zurufe von Pip („Hast du Kristella gesehen? Ist sie wirklich aus Eis? Ganz?“),
Lotte („Kräutertee aus den Bergen – jetzt auch bei mir!“) und Tüftel („Steigkrallen-Version zwei. Bald drei.“).

## 8. Neue Technik für die Inhalte

- Wirkung **`warm`** für Verbrauchsgegenstände (Kälte-Leiste leeren).
- Laden `flocke` in `shops.toml`; Gegenstände in `items.toml`.
- Merker **`eisgriff.stark`**: doppelte Haftdauer und Hochziehen an der Wand (Technik in M2.4.7).
- Gesprächsdateien in `data.rs` (`dialogs!`) eintragen; Figuren Flocke, Kletterer, Kristella als Grafiken aus den Entwürfen.
