# Roadmap Release 2 (Entwurf)

Status: **Entwurf zur gemeinsamen Ausarbeitung** · Schwerpunkte: E-202

## Ziele

1. **Spieler & Gemeinschaft:** Restpunkte aus Release 1 (Credits, Playtests, Balancing), Demos und Replays, Zuschauer-Kamera, Remote-Konsole.
2. **Mehr Spielinhalt:** weitere Waffen, neue Karten und Themen, Sounds in Karten, Musik.
3. **Mitspieler-Bots:** Computergegner, die richtig mitspielen – für Training, volle Server mit wenigen Leuten und als Grundlage für Gegner und NPCs im Abenteuer.
4. **Rollenspiel-Abenteuer:** Einzelspieler mit Geschichte, NPCs und Rollenspiel-Elementen (Leveln, Waffen und Fähigkeiten ausbauen …), zusätzlich als Spielmodus.

## Abhängigkeiten

```
Restpunkte R1 ─► Bots ─────────────┐
                                   ├─► Abenteuer-Grundlage ─► Geschichte & Inhalte ─► Abenteuer als Spielmodus
Waffen & Inhalte ──────────────────┘        (Fortschritt, Spielstände, NPCs, Dialoge, Aufgaben, Editor)
Gemeinschaft (Demos, Zuschauer, Remote-Konsole) läuft parallel
```

- **Bots zuerst:** Gegner und NPCs im Abenteuer brauchen dieselbe Steuerung (Wegfindung auf Tile-Karten, Kampfverhalten, Hook-Nutzung). Bots liefern Eingaben wie Spieler (E-035) – Physik und Netz bleiben unverändert.
- **Abenteuer-Grundlage vor Inhalten:** Fortschrittssystem, Spielstände, NPCs mit Dialogen, Aufgaben und die Editor-Werkzeuge dafür, bevor Geschichte und Karten entstehen.
- **Spielgefühl bleibt Kern:** Rollenspiel-Werte dürfen das Bewegungs- und Hook-Gefühl nicht verwässern; Werte wie Schaden oder Leben im Abenteuer sind eigene Tuning-Sätze.

## Meilensteine (Vorschlag, Reihenfolge offen)

| # | Meilenstein | Inhalt |
|---|---|---|
| R2-M1 | Restpunkte Release 1 | Credits-Seite, Playtests und Balancing, Pakete auf Windows/macOS geprüft, ggf. 0.9.x |
| R2-M2 | Bots | Wegfindung, Kampf- und Hook-Verhalten, Schwierigkeitsstufen, Bots auf Servern und im Training |
| R2-M3 | Waffen & Inhalte | weitere Waffen, Sounds in Karten, neue Themen und Karten |
| R2-M4 | Abenteuer-Grundlage | Fortschritt (Erfahrung, Stufen, Ausbau), Spielstände, NPCs, Dialoge, Aufgaben, Editor-Erweiterungen |
| R2-M5 | Geschichte & Abenteuer | Welt, Kapitel, Figuren, Gegner, Bosse – nach den Vorgaben des Projektinhabers |
| R2-M6 | Abenteuer als Spielmodus | Mehrspieler-Fassung (z. B. Koop) |
| R2-M7 | Gemeinschaft | Demos und Replays, Zuschauer-Kamera, Remote-Konsole |
| R2-M8 | Release 2 | Abnahme, Pakete, Veröffentlichung |

## Zu klären (gemeinsam)

Siehe offene Punkte O-200 bis O-207 in [`entscheidungen.md`](entscheidungen.md).
