# Abenteuer-Inhalte schreiben

Stand: R2-M1 (A1.9) · Entscheidungen: E-217, E-218, E-246 bis E-251

Alle Inhalte des Abenteuers sind TOML-Dateien unter `assets/adventure/`. Texte stehen immer in **beiden Sprachen nebeneinander** (`{ de = "…", en = "…" }`); fehlt eine, meldet das Spiel beim Laden Datei und Stelle (ebenso unbekannte Figuren, Gegenstände, Aufgaben, Knoten und Tippfehler in Bedingungen). `cargo xtask check` prüft die mitgelieferten Inhalte.

| Datei | Inhalt |
|---|---|
| `characters.toml` | Figuren mit Namen und Bild (`elora` braucht keinen Eintrag) |
| `dialogs/<id>.toml` | ein Gespräch je Datei (neue Dateien in `crates/elora-adventure/src/data.rs` bei `dialogs!` eintragen) |
| `quests.toml` | Aufgaben mit Schritten |
| `items.toml`, `skills.toml`, `upgrades.toml`, `shops.toml`, `progression.toml`, `creatures.toml` | Gegenstände, Fähigkeitenbaum, Waffen-Ausbau, Läden, Fortschritt, Gegner ([`fortschritt.md`](../release-2/fortschritt.md)) |

## Gespräche

```toml
speaker = "oma"                       # Standard-Sprecher

[[start]]                             # Einstiege: der erste, dessen Bedingung gilt
if = "quest brunnen aktiv"
node = "erinnerung"

[[start]]
node = "begruessung"

[[node]]
id = "begruessung"
text = { de = "Ach, Elora …", en = "Oh, Elora …" }
do = ["zuneigung oma +1"]             # Folgen beim Erreichen (freiwillig)
next = "weiter"                       # ohne Antworten: nächster Knoten; ohne next = Ende

[[node.choice]]                       # Antworten (freiwillig)
tone = "freundlich"                   # freundlich | neugierig | frech (freiwillig)
if = "stufe >= 2"                     # nur sichtbar, wenn die Bedingung gilt
text = { de = "…", en = "…" }
next = "zusage"                       # ohne next = Ende
do = ["quest brunnen start"]

[[bark]]                              # kurze Zurufe als Sprechblase (E-222)
if = "quest brunnen aktiv"
text = { de = "Pass auf dich auf!", en = "Take care!" }
```

Ein Knoten kann `speaker = "elora"` oder eine andere Figur haben. Jeder Knoten muss erreichbar sein.

**Tasten im Text (A1.9):** `{taste:<aktion>}` zeigt die belegte Taste, z. B. `{taste:jump}`, `{taste:interact}`, `{taste:hook}`, `{taste:fire}`, `{taste:down}`, `{taste:quick_heal}`, `{taste:scoreboard}` (Namen wie in den Einstellungen-Dateien der Steuerung). Gilt in Knoten, Antworten und Zurufen.

**Erscheinen:** `show_if = "<Bedingung>"` in `characters.toml` zeigt eine Figur nur, solange die Bedingung gilt (z. B. die Hummel erst nach dem Kampf: `merker besiegt.brummbaer`). Besiegte Hüter setzen den Merker `besiegt.<art>`.

**Begleiter (E-308):** `follower = "<gegnerart>"`, `follow_if = "<Bedingung>"` und `home_zone = "<zone>"` in `characters.toml`: Solange die Bedingung gilt, folgt die Gegnerart (Verhalten `follower`) Elora, auch über Kartenwechsel. Erreicht sie die Zone, gilt der Merker `<id>.daheim`.

**Stimme (E-286):** In `characters.toml` setzt `voice` die Tonhöhe der Plapperlaute, während der Text erscheint (1 = mittel, kleiner = tiefer, 0 = stumm). Elora spricht mit 1,25.

**Musik (E-285, E-290):** In `worldmap.toml` wählt `music = "<name>"` je Gebiet die Datei `assets/music/<name>.ogg` (Ogg Vorbis, 44,1 kHz). Beim Wechsel des Gebiets wird übergeblendet; `menu.ogg` läuft im Hauptmenü. Quelle und Lizenz gehören in `assets/SOURCES.md`.

**Schilder (E-273):** Wegweiser sind Figuren mit `fixed = true` in `characters.toml` (drehen sich nicht zu Elora); jedes Schild hat ein eigenes Gespräch `dialogs/schild-<ort>.toml`.

## Bedingungen (`if`)

| Bedingung | Bedeutung |
|---|---|
| `stufe >= 3`, `glanz < 50` | Stufe, Glanztropfen; Vergleiche `= != < <= > >=` |
| `quest brunnen neu` / `aktiv` / `erledigt` / `gescheitert` | Zustand einer Aufgabe |
| `quest brunnen schritt bruecke` | aktueller Schritt einer Aufgabe |
| `merker oma.frech`, `merker tor >= 2` | Weltzustand (ohne Vergleich: gesetzt) |
| `zuneigung lotte >= 5` | Zuneigung einer Figur (−10 bis 10) |
| `hat bernstein 3`, `hat heiltrank` | Gegenstand (ohne Zahl: mindestens einer) |
| `faehigkeit gleiten` | hook-ruck, heranhooken, stampfen, eisgriff, gleiten |

Verknüpfen mit ` und `, verneinen mit `nicht ` davor: `nicht merker oma.frech und stufe >= 2`.

## Folgen (`do`)

| Folge | Bedeutung |
|---|---|
| `quest brunnen start` / `weiter` / `fertig` / `scheitern` | Aufgabe beginnen, aktuellen Schritt abschließen, ganz abschließen, scheitern lassen |
| `zuneigung oma +1` | Zuneigung ändern |
| `merker oma.frech = 1`, `merker tor +1` | Weltzustand setzen oder ändern |
| `gib heiltrank 2`, `nimm bernstein 3` | Gegenstand geben bzw. abnehmen (auch `glanztropfen`) |
| `erfahrung 50`, `punkte 1` | Erfahrung, Tautropfen-Punkte |
| `faehigkeit hook-ruck`, `waffe granate` | Gebietsfähigkeit oder Waffe freischalten |
| `laden lotte`, `schmied`, `baum` | Laden, Schmiede, Fähigkeitenbaum öffnen |

## Aufgaben

```toml
[[quest]]
id = "brunnen"
kind = "main"                          # main | side
giver = "oma"
name = { de = "…", en = "…" }
desc = { de = "…", en = "…" }
reward = { xp = 50, glanztropfen = 30, items = [{ item = "heiltrank", count = 1 }], points = 0 }
fail_if = "merker brunnen.zu_spaet"    # freiwillig: scheitert, sobald das gilt (E-250)
next = "bluetenquelle"                 # freiwillig: beginnt nach dem Abschluss

[[quest.step]]
id = "tueftel"
text = { de = "Bei Tüftel vorbeischauen", en = "Drop by Tüftel's workshop" }
goal = { type = "talk", who = "tueftel" }
```

| Ziel (`goal.type`) | Felder | erledigt, wenn … |
|---|---|---|
| `talk` | `who` | ein Gespräch mit der Figur beginnt |
| `reach` | `map`, optional `zone` | Elora die Karte bzw. Zone erreicht (Zonen kommen mit A1.5) |
| `defeat` | `kind`, `count`, optional `map` | so viele Gegner der Art besiegt sind |
| `collect` | `item`, `count` | Elora so viele besitzt |
| `bring` | `item`, `count`, `to` | Elora sie der Figur bringt (sie werden abgegeben) |
| `flag` | `flag`, optional `value` (1) | der Merker den Wert hat (Schalter, Truhe, Tür …) |
| `manual` | – | ein Gespräch `quest <id> weiter` ausführt |

Das Aufgabenbuch zeigt erledigte Schritte und den aktuellen, weitere als „?“ (E-251).

## Abenteuer-Karten im Editor (A1.8)

Werkzeug **9 „Abenteuer“** (E-268): Art in der Seitenleiste wählen, Klick setzt das Objekt auf den Boden unter der Maus (Sammelstücke schweben in der Tile-Mitte), Tür, Übergang, Zone und Kamera werden aufgezogen. Klick auf ein Objekt wählt es, Ziehen verschiebt (auf ganze Tiles), Rechtsklick oder Entf löscht; Rückgängig wie gewohnt. Rechts stehen die Werte des gewählten Objekts, bei NPCs die Vorschau des Gesprächs und **„Gespräch testen“** (Fenster mit Änderungen an Merkern, Aufgaben, Zuneigung und Gegenständen, E-270). „Inhalte neu laden“ liest `assets/adventure` ohne Neustart; die **Prüfung** zeigt fehlende Gegnerarten, Figuren, Gespräche, Gegenstände, falsche Bedingungen und Übergänge ohne Ziel.

- **Kartenname** = Dateiname und Ziel von Übergängen (z. B. `wiese-1`). Gespeichert wird nach `<Benutzerverzeichnis>/maps/abenteuer/<name>.emap`; das Spiel nimmt diese Datei vor der mitgelieferten gleichen Namens (E-271).
- **F5** testet im Abenteuer mit dem **Teststand** (Stufe, Fähigkeiten, Waffen, Merker wie `tor.dorf=1, oma.frech`; Start an einem Eingang/Quellstein oder an der Maus). Es wird nichts gespeichert; Übergänge laden die anderen Karten, die gerade bearbeitete auch ungespeichert. Esc kehrt in den Editor zurück (E-269).
