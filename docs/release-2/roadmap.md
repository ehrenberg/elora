# Roadmap Release 2

Status: **Reihenfolge festgelegt (E-216)** · Schwerpunkte: E-202

## Ziele

1. **Spieler & Gemeinschaft:** Restpunkte aus Release 1 (Credits, Playtests, Balancing), Demos und Replays, Zuschauer-Kamera, Remote-Konsole.
2. **Mehr Spielinhalt:** weitere Waffen, neue Karten und Themen, Sounds in Karten, Musik.
3. **Mitspieler-Bots:** Computergegner, die richtig mitspielen – für Training, volle Server mit wenigen Leuten und als Grundlage für Gegner und NPCs im Abenteuer.
4. **Rollenspiel-Abenteuer:** Einzelspieler mit Geschichte in einer **Hub-Welt** (Dorf + freischaltbare Gebiete, E-203); Stufen & Fähigkeitenbaum, Waffen-Ausbau, Ausrüstung & Beute, Aufgaben & Dialoge (E-206). Dazu ein **Rollenspiel-PvP-Modus**, in dem man während des Matches levelt (E-204). Geschichte: Claude schlägt vor, der Projektinhaber entscheidet (E-205).

## Abhängigkeiten

```
Abenteuer-Grundlage ─► Geschichte & Abenteuer ─► Rollenspiel-PvP-Modus
        │  (Fähigkeiten, Gegner, Fortschritt, Spielstände, Dialoge, Aufgaben, Editor)
        └─► Mitspieler-Bots (nutzen Gegner-Steuerung und Wegfindung)
Waffen & Inhalte, Gemeinschaft laufen dazwischen
```

- **Abenteuer-Grundlage zuerst** (E-216): Technik und Editor-Werkzeuge, bevor Geschichte und Karten in Menge entstehen.
- **Bots danach:** Die Mehrspieler-Bots bauen auf Steuerung und Wegfindung der Abenteuer-Gegner auf; Bots liefern Eingaben wie Spieler (E-035).
- **Spielgefühl bleibt Kern:** Rollenspiel-Werte dürfen das Bewegungs- und Hook-Gefühl nicht verwässern; Werte wie Schaden oder Leben im Abenteuer sind eigene Tuning-Sätze.
- **Restpunkte Release 1** (Playtests, Balancing, Pakete auf Windows/macOS) übernimmt der Projektinhaber nebenher (O-51).

## Meilensteine

| # | Meilenstein | Inhalt |
|---|---|---|
| R2-M1 | Abenteuer-Grundlage | Fähigkeiten, Gegner, Fortschritt, Spielstände, NPCs, Dialoge, Aufgaben, Editor; Abnahme mit dem Prolog – Plan: [`a1-plan.md`](a1-plan.md) |
| R2-M2 | Geschichte & Abenteuer | Tauwinkel, fünf Gebiete, Bosse, Finale, Nebenaufgaben ([`weltbuch.md`](weltbuch.md)); Teil-Meilensteine je Kapitel: M2.1 Blütenwiesen ([`m2-1-plan.md`](m2-1-plan.md)), M2.2 Murmelwald ([`m2-2-plan.md`](m2-2-plan.md)), M2.3 Glutsandwüste ([`m2-3-plan.md`](m2-3-plan.md)), M2.4 Frostspitzen ([`m2-4-plan.md`](m2-4-plan.md)), M2.5 Sternschlucht, M2.6 Finale |
| R2-W1 ✅ | Wetter | Regen, Gewitter, Nebel, Wind, Sandsturm, Schnee im Abenteuer und als Karteneigenschaft; leichte Spielwirkung im Abenteuer (E-329 bis E-335) – **abgenommen** (E-339) – Plan: [`w1-plan.md`](w1-plan.md) |
| R2-RF | Refactoring | Before the next release: English throughout (E-348) and cleanup of large files, duplicated logic and lint allows – plan: [`refactoring-plan.md`](refactoring-plan.md) |
| R2-M3 | Mitspieler-Bots | Wegfindung, Kampf- und Hook-Verhalten, Schwierigkeitsstufen, Bots auf Servern und im Training |
| R2-M4 | Waffen & Inhalte | weitere Waffen, Sounds in Karten, neue Themen und Karten |
| R2-M5 | Rollenspiel-PvP-Modus | „Quellenkampf“: Stufen, Ausbau und Beute während des Matches (E-204) |
| R2-M6 | Gemeinschaft | Demos und Replays, Zuschauer-Kamera, Remote-Konsole |
| R2-M7 | Release 2 | Abnahme, Pakete, Veröffentlichung |

## Zu klären (gemeinsam)

Siehe offene Punkte O-200 bis O-207 in [`entscheidungen.md`](entscheidungen.md).

Idee für die Zeit nach Release 2: **dauerhafte Welt** auf einem Server (O-208).
