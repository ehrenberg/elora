# R2-W1 – Wetter – Umsetzungsplan

Status: **Freigegeben, in Umsetzung** (E-336) · Entscheidungen E-329 bis E-336 · Grundlage: O-209 (Tageszeiten bleiben offen, E-332), E-320 (Hitze), E-328 (Post-Shader, Farbe der Quellen)

## Ziel

Wetter macht die Gebiete lebendiger und erzählt mit: Solange eine Quelle schweigt, ist ihr Gebiet
trüb; nach der Befreiung klart es auf, und das Wetter wechselt beim Betreten. Im Abenteuer wirkt
es leicht aufs Spiel (Wind, Nässe, Nebel), im Mehrspieler ist es reine Stimmung. Karten bekommen
Wetter als Eigenschaft im Editor.

**Abnahme:** jedes Wetter einmal sehen und hören (Wetter-Testkarte), Tauwinkel und Kapitel 1–3
mit Wetter anspielen, eine Mehrspieler-Karte mit Wetter online.

## Ausgangslage

| Baustein | Stand | Nutzen |
|---|---|---|
| Post-Shader | Hitzeflimmern, Sättigung (E-320, E-328) | Farbstimmung, Abdunkeln, Nebel, Blitze |
| Partikel | Staub, Funken, Sporen (`effects.rs`) | Vorlage für Wetterpartikel (eigene Schicht, weil viele) |
| Himmel, Hintergründe | Verlauf und Färbung je Karte | trüber Himmel, dichtere Wolken |
| Deko-Animationen | Wind in Bäumen, Fahnen | stärkerer Wind |
| Musik | gestreamt je Gebiet (kira) | Vorlage für eine zweite, durchlaufende Umgebungsspur |
| Kartenformat `.emap` | Abschnitte; unbekannte werden übersprungen | neuer Abschnitt `WTHR` bleibt mit 0.9.1 verträglich |
| Simulation | deterministisch, gleich im Mehrspieler | Spielwirkung nur, wenn `world.adventure` (E-330) |

## Wetterarten (E-333)

| Wetter | Optik | Klang | Wirkung im Abenteuer (E-330) |
|---|---|---|---|
| **Schön** | wie bisher | – | – |
| **Regen** (Niesel bis kräftig) | schräge Tropfen nach Wind, Spritzer auf Oberflächen, Himmel grauer, Bild leicht abgedunkelt | Regen | Boden nass: Bremsen etwas weicher (ein Drittel des Eis-Effekts) |
| **Gewitter** | wie Regen, dunkler; Blitze hellen kurz das ganze Bild auf, Wolkenleuchten | Regen, Donner (passend zum Blitz verzögert) | wie Regen, dazu Wind in Böen; **Blitzeinschläge** mit Warnung am Boden, kleiner Schaden (E-336) |
| **Nebel** | Schleier über der Welt, nach unten dichter; ferne Hintergründe verschwinden | leiser Wind | Sichtweite: die Kamera zeigt weniger weit voraus, Gegner tauchen aus dem Schleier auf |
| **Wind** (mit Blättern oder Blüten) | Blätter bzw. Blütenblätter wirbeln, Bäume und Fahnen wiegen stärker | Wind | Wind schiebt Elora in der Luft und Granaten (am Boden kaum) |
| **Sandsturm** | Sandschleier waagerecht, Bild gelb-braun, Sicht kürzer | Sturm, rieselnder Sand | Wind wie oben, stärker; Sicht kürzer wie Nebel |
| **Schnee** | ruhige Flocken, Himmel hell-grau | sehr leise | Boden etwas rutschig (wie Regen) |
| **Schneesturm** | dichte schräge Flocken, Sicht kurz | Sturm | Wind stark, Boden rutschig, Sicht kürzer |

Stärke je Wetter 0–1 (Niesel bis Platzregen); Wind mit Richtung und Böen. Die Hitze der Wüste
(E-320) bleibt: Bei Sandsturm füllt die Sonne die Hitze-Leiste nicht (Sand verdeckt die Sonne).

## Wetter je Gebiet (Vorschlag, E-331)

| Gebiet | solange die Quelle schweigt | danach beim Betreten zufällig |
|---|---|---|
| Tauwinkel | Nieselregen (nach der 1. Quelle nur noch selten) | schön (meist), Wind mit Blüten, Sommerregen |
| Blütenwiesen | Regen, ab und zu Gewitter | schön, Wind mit Blüten, Sommerregen |
| Murmelwald | Nebel, Regen | schön, Nebel am Morgen, Wind mit Blättern |
| Glutsandwüste | Sandsturm, Hitzegewitter | schön (heiß), Sandsturm selten |
| Frostspitzen (Kapitel 4) | Schneesturm | Schnee, schön |
| Arenen der Hüter | festes Wetter je Arena (z. B. Gewitter über der Blütenquelle?) – oder schön, damit der Kampf lesbar bleibt | schön |

Gewichte und Stärken stehen als Daten in `assets/adventure/worldmap.toml`; das Wetter wird beim
Betreten einer Karte gewürfelt und gilt bis zum Kartenwechsel. Eine Karte mit eigenem Wetter
aus dem Editor gilt vor den Gebietsregeln (z. B. Höhlen: kein Regen).

## Mehrspieler-Karten (E-329)

Wetter ist eine Karteneigenschaft (Art, Stärke, Wind) und wird mit der Karte übertragen; jeder
Client zeichnet es selbst. Keine Spielwirkung, kein neues Protokoll. Vorschlag für die
Release-Karten: `dm-winter` Schnee, `ctf-nacht` leichter Nebel, `dm-wueste` Wind mit Sand, die
übrigen schön – zu entscheiden mit der Freigabe.

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| W1.0 ✅ | Entwürfe | Stimmungsbilder: dieselbe Szene in allen acht Wettern (Python-Skript), dazu Partikelformen (Tropfen, Flocke, Blatt, Blüte, Sandkorn) | Deine Auswahl |
| W1.1 ✅ | Datenmodell | Wetter-Typ (Art, Stärke, Wind, Böen) in `elora-map`; Abschnitt `WTHR` im Kartenformat; Eigenschaft im Editor mit Vorschau; Wetter-Testkarte | Tests (Format hin und zurück, alte Karten unverändert) |
| W1.2 ✅ | Optik | Wetterpartikel in zwei Ebenen (hinter und vor der Spielfläche), Spritzer und Flocken auf Oberflächen, Blitze; Post-Shader: Farbstimmung, Abdunkeln, Nebel nach Höhe, Sandschleier; Himmel trüber; Wind lässt Deko stärker wiegen; Einstellung „Wetter: voll, sanft, aus“ (E-335) | Sichtprüfung, Bildrate bei vollem Regen |
| W1.3 ✅ | Abenteuer-Steuerung | Gebietsregeln in `worldmap.toml` (trüb solange die Quelle schweigt, sonst gewichteter Zufall), Wahl beim Betreten, Übergänge weich; Sandsturm verdeckt die Sonne (Hitze) | Tests |
| W1.4 ✅ | Spielwirkung | in der Simulation nur mit `world.adventure`: Windkraft auf Elora in der Luft und Granaten, nasser/verschneiter Boden (weichere Reibung), Blitze mit Warnung und Schaden im Gewitter (E-336), Sichtweite der Kamera bei Nebel/Sturm; Werte als Tuning (A-29 bis A-35) | Tests; Golden-Tests unverändert |
| W1.5 ✅ | Klang | zweite Umgebungsspur mit weichem Überblenden; Regen, Wind, Sturm, Sand, Donner aus freien Quellen (CC0, zum Anhören vorgelegt, E-334, Auswahl E-338) | Deine Hörprobe |
| W1.6 ✅ | Karten | Release-Karten: `dm-winter` Schnee (0,6), `ctf-nacht` leichter Nebel (0,35), `dm-wueste` Wind mit Sand (0,3), übrige schön; Hüter-Arenen bleiben schön (D-W1-01); in Höhlen und unter Dächern kein Niederschlag, Wetterklänge gedämpft | Sichtprüfung |
| W1.7 | Abnahme | Wetter-Testkarte, Kapitel 1–3, eine Mehrspieler-Runde | Deine Abnahme |

## Prüfliste Abnahme (W1.7)

Wetter umschalten: Debug-Panel (F1) → „Wetter“ (Art, Stärke, Wind); im Abenteuer wirkt das auch
auf Wind, Nässe und Blitze.

| # | Wo | Was prüfen |
|---|---|---|
| 1 | `elora maps/wetter-test.emap` | alle neun Wetter nacheinander: Partikel, Farbstimmung, Nebel, Blitze; unter dem Dach und in der Grube kein Niederschlag; Klang unter dem Dach leiser |
| 2 | dieselbe Karte | Einstellung Grafik → Wetter „voll / sanft / aus“; Bildrate bei vollem Regen und Schneesturm (Debug-Panel) |
| 3 | Kapitel 1 (Blütenwiesen) | Wetter beim Betreten wechselt mit der Zeit; Gewitter: Boden glimmt, Blitz schlägt ein und schadet, Donner; Ausweichen gelingt |
| 4 | Kapitel 2 (Murmelwald) | Nebel: Kamera schaut weniger weit voraus, Spiel bleibt lesbar; Blätter-Wind: Elora treibt beim Springen über Gruben spürbar, aber beherrschbar |
| 5 | Kapitel 3 (Glutsandwüste) | Sandsturm: Hitze-Leiste füllt sich nicht (Schatten), Granaten treiben mit dem Wind; Arena bleibt schön |
| 6 | Regen/Schnee im Abenteuer | nasser Boden: weicheres Bremsen, an Kanten nicht unfair |
| 7 | Mehrspieler | `dm-winter`, `ctf-nacht`, `dm-wueste` mit Server: Wetter sichtbar und hörbar, keine Spielwirkung |

## Technische Festlegungen (Vorschlag)

- **Partikel als eigene Schicht** (nicht im allgemeinen Effektsystem): feste Obergrenze (voll etwa
  600, sanft 200), am Kameraausschnitt verankert und mit leichtem Parallax, damit Regen nicht
  „mitläuft“. Gezeichnet als Linien (Regen, Sand) bzw. kleine Formen (Flocken, Blätter).
- **Shader:** ein zweiter Parametersatz für Farbstimmung (Tönung, Helligkeit), Nebel (Farbe,
  Dichte, Höhenverlauf) und Blitz; derselbe Durchgang wie Hitzeflimmern und Sättigung.
- **Spielwirkung deterministisch** in `elora-sim`: Wetter der Welt wird von der Sitzung gesetzt
  (wie Hitze), Böen aus einem festen Pseudo-Zufall nach Tick – Aufzeichnungen bleiben gleich.
- **Sichtweite** ist Darstellung (Kamera und Nebel), keine Simulation.
- **Mehrspieler:** nur der Kartenabschnitt; Protokollversion bleibt 6.
- **Leistung:** Partikel als ein Mesh je Frame; bei „sanft“ weniger Partikel und keine Blitze.

## Offen zur Freigabe

| # | Frage | Vorschlag |
|---|---|---|
| D-W1-01 | Wetter in den Hüter-Arenen | schön, damit Kämpfe lesbar bleiben; höchstens leichter Wind |
| D-W1-02 | Release-Karten mit Wetter | `dm-winter` Schnee, `ctf-nacht` leichter Nebel, `dm-wueste` Wind mit Sand |
| D-W1-03 | Stärke der Spielwirkung | Wind: Elora in der Luft bis etwa 1/6 der Luftsteuerung, Granaten spürbar; Nässe: ein Drittel des Eis-Effekts |
| D-W1-04 | Blitze | **E-336: Blitze können schaden** – nur im Abenteuer: kurz glimmt der Boden an der Einschlagstelle (Warnung), dann schlägt der Blitz ein (kleiner Schaden im Umkreis); im Mehrspieler nur Optik und Klang |
