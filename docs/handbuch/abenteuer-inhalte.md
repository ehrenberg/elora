# Abenteuer-Inhalte schreiben

Stand: R2-M1 (A1.4) · Entscheidungen: E-217, E-218, E-246 bis E-251

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
