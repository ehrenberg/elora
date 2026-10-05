# R2-M2.3 – Inhalte von Kapitel 3 (Entwurf, M2.3.4)

Status: **Freigegeben und eingebaut** (Texte freigegeben 2026-10-06) · Grundlage: [`m2-3-plan.md`](m2-3-plan.md), [`weltbuch.md`](weltbuch.md) §4.3 und §5, E-243, E-316 bis E-325

Deutsche Texte; die englischen kommen beim Einbau dazu. Bedingungen und Folgen in der Schreibweise
der Inhaltsdateien ([`../handbuch/abenteuer-inhalte.md`](../handbuch/abenteuer-inhalte.md)).

## 1. Figuren

| Id | Name | Wo | Stimme | Wesen |
|---|---|---|---|---|
| `sirup` | Karawanenführer Sirup | Karawanenlager (`wueste-2`) | tief, gemütlich (0,85) | weitgereist, erzählt gern und ausschweifend, handelt mit Augenzwinkern |
| `palma` | Oasen-Hüterin Palma | an der Oase (`wueste-2`) | hell (1,15) | ruhig, sorgt sich um ihre Oase, spricht in Bildern vom Wasser |
| `schlange` | Sandschlange | Arena, nach dem Kampf (`show_if = "merker besiegt.sandschlange"`) | sehr tief (0,5) | müde, verlegen, warmherzig |
| `tafel-1`, `tafel-2`, `tafel-kammer` | Ruinentafel | `wueste-3`, Kammer | stumm, fest | Steintafeln des Glutvolks |
| `ruinenquelle` | Ruinenquelle | `wueste-3` | stumm, fest | kleine Quelle zwischen den Säulen (Schlauch füllen) |
| `giessstelle-1` bis `-3` | Verdorrte Stelle | Oase | stumm, fest | trockene Erde; nach dem Gießen blüht sie (Deko `-verdorrt` → `-befreit`) |

## 2. Hauptaufgabe „Spuren im Sand“ (`glutsand`, ersetzt die Ankündigung)

> Die dritte Quelle liegt in der Glutsandwüste. Eine Karawane erzählt von einem grauen Wanderer.

| Schritt | Text | Ziel |
|---|---|---|
| `aufbruch` | Am Ostpfad den Hohlweg in die Wüste nehmen | `wueste-1` erreichen |
| `sirup` | Die Karawane in der Wüste finden | mit Sirup sprechen |
| `ruinen` | Den grauen Spuren zu den Ruinen folgen | `wueste-3` erreichen |
| `quelle` | Die Glutquelle im Sandkessel erreichen | `wueste-arena` erreichen |
| `hueter` | Die Sandschlange beruhigen | Sandschlange besiegen |
| `funke` | Tüftel den Quellfunken bringen | Quellfunke zu Tüftel |
| `fest` | Mit Oma Pfütze am Brunnen feiern | mit Oma sprechen |

Lohn: 350 Erfahrung, 160 Glanztropfen. Danach beginnt die Ankündigung **„Der Ruf der Frostspitzen“** (Schritt von Hand, „bald“).

## 3. Nebenaufgaben

### „Wasser für die Oase“ (`oase`, von Palma, E-319, E-322)

| Schritt | Text | Ziel |
|---|---|---|
| `fuellen` | Den Wasserschlauch an der Ruinenquelle füllen | 3 Schluck Wasser haben |
| `giessen` | Die drei verdorrten Stellen der Oase gießen | Merker `oase.gegossen` = 3 |
| `danke` | Palma von der Oase erzählen | mit Palma sprechen |

Palma gibt den leeren **Wasserschlauch**. An der Ruinenquelle: Schlauch voll = **3 Schluck Wasser**. Jede
verdorrte Stelle nimmt einen Schluck, blüht auf und zählt `oase.gegossen` hoch. Lohn: 80 Erfahrung,
1 Tautropfen-Punkt, 3 Kaktusfrüchte.

### „Die verschüttete Ruine“ (`ruine`, von Sirup, E-323)

| Schritt | Text | Ziel |
|---|---|---|
| `kammer` | Einen Weg in die verschüttete Kammer finden | Zone `ruinenkammer` in `wueste-3` (Bröckelboden, nur mit Stampfen) |
| `tafel` | Die Tafel in der Kammer lesen | mit `tafel-kammer` sprechen |
| `bericht` | Sirup von der Kammer erzählen | mit Sirup sprechen |

In der Kammer steht eine Truhe mit dem **Sonnenschleier**. Lohn bei Sirup: 80 Erfahrung, 1 Tautropfen-Punkt,
60 Glanztropfen. Vor dem Kampf sieht man die Kammer nur durch einen Spalt; Sirup sagt, man brauche „jemanden,
der kräftig aufstampfen kann“.

## 4. Gegenstände und Sirups Laden (E-324)

| Id | Art | Name | Wirkung | Preis |
|---|---|---|---|---|
| `wasserschlauch` | Schlüssel | Wasserschlauch | „Aus Ziegenleder, riecht ein bisschen nach Kamel.“ | – |
| `wasser` | Schlüssel | Schluck Wasser | „Klar und kühl, aus der Ruinenquelle.“ | – |
| `kaktusfrucht` | Verbrauch | Kaktusfrucht | heilt 3 und **leert die Hitze-Leiste** (neu) | 12 |
| `sonnenschleier` | Hut | Sonnenschleier | **Hitze-Leiste füllt sich 40 % langsamer** (neu) | – (Lohn) |
| `karawanenkette` | Schmuck | Karawanenkette | Schutz nach Treffer +300 ms | 180 |

Sirups Laden: Kaktusfrucht, Glutstein (Material für den Laser-Ausbau, 25), Karawanenkette;
Rabatt ab Zuneigung 5 (10 %) wie bei Lotte. Wüstengegner lassen auch Glutstein fallen (Chance 25 %),
die Sandschlange 3 Glutstein statt Bernstein.

## 5. Gespräche

### Sirup

**Begrüßung** (erster Besuch):
> Na sieh mal an, ein Tropfen in der Glut! Willkommen in Sirups Karawane – Waren aus allen Ecken des Taulands, frisch, selten und nur ein kleines bisschen sandig.

- *(neugierig)* „Hast du hier etwas Seltsames gesehen?“ → **Wanderer**
- *(freundlich)* „Was verkaufst du denn?“ → Laden
- *(frech)* „Nur ein bisschen sandig? Dein Turban ist voller Sand.“ → „Hoho! Der Turban ist ein Erbstück. Der Sand auch.“ (Zuneigung +1) → **Wanderer**

**Wanderer** (setzt Schritt `sirup` fort):
> Seltsam? Vor drei Nächten saß ich am Feuer, da kam einer aus den Dünen. Ganz grau, wie Asche, die laufen gelernt hat. Er kniete an Palmas Oase und trank. Und trank. Am Morgen war die Oase halb leer und so blass wie mein Großvater nach dem Bad.
>
> Seine Spuren führen zu den alten Ruinen. Grau, als hätte jemand die Farbe aus dem Sand gesogen. Ich gehe da nicht hin. Aber du hast so einen … unternehmungslustigen Blick.

- „Ich folge den Spuren.“ → Ende
- „Was sind das für Ruinen?“ → **Ruine** (startet die Nebenaufgabe)

**Ruine:**
> Das Glutvolk hat dort früher die Glutquelle gehütet. Unter den Ruinen liegt eine verschüttete Kammer – mein Vater schwor, da drin steht eine Tafel, die keiner je gelesen hat. Der Boden ist alt und mürbe. Man bräuchte jemanden, der kräftig aufstampfen kann.

- *(freundlich)* „Ich schaue nach, wenn ich kann.“ → `quest ruine start`
- „Vielleicht später.“

**Bericht** (Schritt `bericht`):
> Eine Tafel? Was stand drauf? … Er suchte ein Lied? Hm. Dann ist er vielleicht gar nicht böse, sondern nur verloren. Wie ich, als ich zum ersten Mal ohne Karte losgezogen bin. Hier, für deine Mühe. Und komm wieder – Sirup vergisst keinen Freund.

**Danach / Zurufe:** „Frisch aus dem Norden: Kaktusfrüchte! Na gut, aus dem Süden.“ · „Halt dich im Schatten, Kleines!“

### Palma

**Begrüßung:**
> Leise, bitte. Die Oase schläft. Früher hat sie gesungen wie ein Bach im Frühling. Seit der Graue hier war, wird sie jeden Tag stiller. Drei Stellen am Ufer sind schon ganz verdorrt.

- *(freundlich)* „Kann ich helfen?“ → **Auftrag**
- *(neugierig)* „Wer ist der Graue?“ → „Ich habe ihn nur von weitem gesehen. Er war … traurig, glaube ich. Wer so trinkt, hat großen Durst nach etwas, das Wasser nicht stillen kann.“ → **Auftrag**

**Auftrag:**
> In den Ruinen gibt es noch eine kleine Quelle, tief zwischen den Säulen. Nimm meinen Wasserschlauch. Er fasst genug für alle drei Stellen.

- „Ich bringe Wasser.“ → `gib wasserschlauch 1`, `quest oase start`

**Erinnerung:** „Der Schlauch fasst drei Schluck. Jede verdorrte Stelle braucht einen.“

**Danke** (alle drei gegossen):
> Hörst du das? Sie summt wieder. Ganz leise, aber sie summt. Danke, kleiner Tropfen. Nimm diese Früchte – sie kühlen, wenn die Sonne zu sehr drückt.

**Zurufe:** „Im Schatten der Palmen ist es kühl.“ · „Pssst … die Oase summt.“

### Ruinenquelle (fest)
- ohne Schlauch: „Klares Wasser sprudelt zwischen den Steinen. Wenn man nur etwas hätte, um es mitzunehmen …“
- mit leerem Schlauch: „Du füllst den Wasserschlauch bis zum Rand.“ → `gib wasser 3`
- mit Wasser: „Der Schlauch ist schon voll.“

### Verdorrte Stelle 1–3 (fest)
- ohne Wasser: „Trockene, rissige Erde. Hier wuchs einmal etwas.“
- mit Wasser: „Du gießt einen Schluck Wasser. Die Erde trinkt gierig – und kleine Blüten öffnen sich.“ → `nimm wasser 1`, `merker befreit.giessstelle-N = 1`, `merker oase.gegossen +1`
- danach: „Hier blüht es wieder.“

### Ruinentafeln (fest)
- **Tafel 1:** „Wir, das Glutvolk, hüten das goldene Wasser. Seine Wärme gehört allen, die frieren.“
- **Tafel 2** (Bild: eine graue Gestalt beugt sich über eine Quelle): „Einst kam ein Grauer in der Nacht. Wo er trank, wurde der Sand blass. Wir fürchteten ihn und verschlossen die Kammer.“
- **Kammertafel:** „Doch er trank nicht aus Gier. Er suchte ein Lied, das er verloren hatte – das Lied der Quelle, die vor allen anderen war.“ → Schritt `tafel`

### Sandschlange (nach dem Kampf)
> Sssso … kühl … endlich. Verzeih, kleiner Tropfen. Etwas Graues hat aus meiner Quelle getrunken. Danach war mir so kalt, so schrecklich kalt, dass ich jeden angefaucht habe, der näher kam.
>
> Es ging nach Norden. Zu den Bergen, wo der Schnee nie schmilzt. Nimm den Funken. Und wenn du ihn findest … sei sanft. Er hat sehr kalt geklungen. Kälter als ich.

### Tüftel (Schritt `funke`)
> Ein goldener Funke! Warm wie … nein, wärmer! Halt das mal. Und das. Und – stell dich da hin. Jetzt spring. Und jetzt RUNTER! Siehst du? Stampfen! Damit brichst du mürbe Böden und haust Gegner um. Probier's auf dem Übungsplatz, da hab ich dir was Bröseliges hingelegt.

→ `faehigkeit stampfen`, `nimm quellfunke 1`, `quest glutsand weiter`

### Klonk (nach Kapitel 3, E-243)
> Hmpf. Du kommst aus der Wüste und hast nicht mal Sand in den Ohren. Respekt. Hier. Hab ich aus Glutstein und einer alten Linse gebaut. Ein Laser. Zeig damit auf nichts, was du magst.

→ `waffe laser`, `gib munition_laser 1`, `merker klonk.laser = 1`

### Oma Pfütze
- **Auftrag** (nach dem Fest von Kapitel 2, Quest aktiv): „Die Glutquelle, Kind. Sie hat mir als Mädchen die Hände gewärmt. Nimm am Ostpfad den Hohlweg nach Süden. Und trink genug – die Sonne dort ist nicht so freundlich wie ich.“
- **Unterwegs:** „Die Wüste wird dich nicht beißen. Nur die Krabben.“
- **Fest:** „Gold! Die Häuser leuchten wie Honig in der Abendsonne. Und du, mein mutiger Tropfen … Ein grauer Wanderer, sagst du? Der nach einem Lied sucht? … Ich weiß nicht warum, aber dabei wird mir ganz wehmütig. Die Berge im Norden, die Frostspitzen. Dort wartet die vierte Quelle.“ → `quest glutsand fertig`, `quest frostspitzen start`, Fest

### Schilder
- `schild-wueste` (Gabelung am Ostpfad): „↓ Glutsandwüste – Hohlweg. Wasser mitnehmen!“
- `schild-treibsand`: „Vorsicht, Treibsand! Wer einsinkt: {taste:jump} drücken.“
- `schild-hitze`: „Hitze macht müde. Schatten unter Felsen und Zeltdächern kühlt.“
- `schild-kammer`: „Hier klingt der Boden hohl …“

## 6. Graue Spuren (E-325)

Graue Fußspuren als Deko (`grauspur`) in `wueste-1` bis `wueste-3` und am Rand der Arena: von der Oase zu den
Ruinen und vom Sandkessel nach Norden. Kein Auftritt des Wanderers.

## 7. Neue Technik für die Inhalte

- Wirkung **`cool`** für Verbrauchsgegenstände (Hitze-Leiste leeren) und Bonus **`heat_pct`** für Ausrüstung.
- Laden `sirup` in `shops.toml`.
- Die Gesprächsdateien in `data.rs` (`dialogs!`) eintragen; Deko `giessstelle-verdorrt`/`-befreit` und `grauspur` aus den Entwürfen.
