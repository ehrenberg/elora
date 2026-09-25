# Analyse: Teeworlds

Stand: 2026-09-25 · Quellen: teeworlds.com, Quellcode `github.com/teeworlds/teeworlds` (`src/game/tuning.h`, `datasrc/content.py`)

Ziel dieses Dokuments: festhalten, **was** Teeworlds ausmacht, damit der Klon das gleiche Spielgefühl erreicht. Entscheidungen, wie wir etwas umsetzen, stehen in [`02-entscheidungen.md`](02-entscheidungen.md).

---

## 1. Überblick

| Merkmal | Teeworlds |
|---|---|
| Genre | 2D-Side-Scroller-Shooter, Multiplayer (Arena) |
| Lizenz | Open Source (Code: zlib-ähnliche Teeworlds-Lizenz; Assets: CC-BY-SA) |
| Sprache / Technik | C++, SDL2, OpenGL, eigenes UDP-Netzwerkprotokoll |
| Letzte Version | 0.7.5 (2020). Nachfolger-Community: **DDNet** (basiert auf 0.6-Protokoll) |
| Spieler pro Server | typischerweise 8–16 (max. 16 in Vanilla) |
| Spielfigur | „Tee“ – runder Ball mit Augen und Füßen, Skins/Farben anpassbar |

## 2. Was das „Feeling“ ausmacht

Das Spielgefühl entsteht fast vollständig aus diesen Punkten – sie müssen im Klon **exakt** stimmen:

1. **Deterministische Tick-Physik mit 50 Ticks/s.** Alle Werte (Geschwindigkeit, Beschleunigung, Gravitation) sind pro Tick definiert. Rendering interpoliert zwischen Ticks.
2. **Der Enterhaken (Hook).** Kernmechanik: an Wände hängen, schwingen, Gegner heranziehen. Er bestimmt Bewegung, Taktik und Skill-Ceiling.
3. **Hohe Luftkontrolle + Doppelsprung.** Man kann in der Luft stark lenken, hat einen zweiten Sprung.
4. **Momentum.** Explosionen (Granate, Hammer) schleudern Spieler, „Rocket-Jumps“ mit Granaten sind möglich.
5. **Schnelle Time-to-Kill, schneller Respawn.** 10 HP + 10 Rüstung, Runden sind kurz und hektisch.
6. **Client-seitige Vorhersage (Prediction).** Eigene Bewegung fühlt sich trotz Ping sofort an.
7. **Präzises Tile-Collision-System** mit 32px-Tiles.
8. **Audio-visuelles Feedback:** Cartoon-Stil, Partikel, Screen-Feedback, Emotes, Treffer-Sounds.

## 3. Welt & Kollision

- Karte = Raster aus **Tiles à 32×32 Einheiten**.
- Tile-Arten in Vanilla: **Luft**, **Solid** (Wand), **Death** (tötet), **Unhookable** (Wand, an der der Hook nicht greift).
- Spieler-Hitbox: **28×28** (physische Größe `PhysSize = 28`), gezeichnet größer (~64px).
- Bewegung pro Tick mit Sub-Stepping gegen Tiles (`MoveBox`), damit schnelle Objekte nicht durch Wände tunneln.
- Entities-Layer: Spawnpunkte (neutral/rot/blau), Flaggen-Stände, Pickups (Herz, Rüstung, Waffen, Ninja).

## 4. Bewegungsphysik (Default-Tuning, Einheiten/Tick bei 50 TPS)

| Parameter | Wert | Bedeutung |
|---|---|---|
| `GroundControlSpeed` | 10.0 | Max. Laufgeschwindigkeit am Boden |
| `GroundControlAccel` | 100/50 = 2.0 | Beschleunigung am Boden |
| `GroundFriction` | 0.5 | Reibung am Boden ohne Input |
| `GroundJumpImpulse` | 13.2 | Sprungimpuls vom Boden |
| `AirJumpImpulse` | 12.0 | Doppelsprung-Impuls |
| `AirControlSpeed` | 250/50 = 5.0 | Max. Luftlenk-Geschwindigkeit |
| `AirControlAccel` | 1.5 | Luftbeschleunigung |
| `AirFriction` | 0.95 | Luftreibung |
| `Gravity` | 0.5 | Gravitation pro Tick |
| `VelrampStart` | 550 | ab dieser Geschw. wird Bewegung gedämpft |
| `VelrampRange` | 2000 | Bereich der Dämpfung |
| `VelrampCurvature` | 1.4 | Kurvenform der Dämpfung |
| `PlayerCollision` | 1 | Spieler kollidieren miteinander |
| `PlayerHooking` | 1 | Spieler können sich gegenseitig hooken |

Weitere Details:
- **Doppelsprung:** ein Luftsprung, wird bei Bodenkontakt zurückgesetzt. Visuell sichtbar (Füße).
- **Velocity Ramp:** bei sehr hohen Geschwindigkeiten wird die effektive Bewegung nichtlinear reduziert – verhindert unkontrollierbare Geschwindigkeiten.
- Werte werden **quantisiert** (Positionen/Geschwindigkeiten auf Ganzzahlen bzw. feste Genauigkeit gerundet), damit Server und Client deterministisch gleich rechnen.

## 5. Hook

| Parameter | Wert |
|---|---|
| `HookLength` | 380 (max. Reichweite) |
| `HookFireSpeed` | 80 / Tick |
| `HookDragAccel` | 3.0 |
| `HookDragSpeed` | 15.0 |

Zustände: `Idle → Flying → Grabbed (an Wand oder Spieler) → Retracted`.
- Hook fliegt in Zielrichtung, greift an Solid-Tiles (nicht an Unhookable) oder an Spielern.
- Gegriffen: Spieler wird zur Hook-Position gezogen (Beschleunigung bis `HookDragSpeed`).
- Spieler-Hook: zieht **beide** Spieler zueinander (Kraft auf beide verteilt).
- Player-Hook hat ein Zeitlimit: `SERVER_TICK_SPEED + SERVER_TICK_SPEED/5` = 60 Ticks = **1,2 s**, danach löst er sich.
- Kraftverteilung beim Player-Hook: der gehookte Spieler bekommt `Dir * Accel * 1.5`, der hookende weniger.
- Loslassen der Hook-Taste = Hook zurück.

## 6. Waffen

Jeder Spieler hat immer **Hammer** und **Pistole**. Weitere Waffen per Pickup. Max. Munition 10.

| Waffe | Schaden | Feuerrate (ms) | Munition | Besonderheit |
|---|---|---|---|---|
| Hammer | 3 | 125 | ∞ | Nahkampf, starker Knockback (schleudert Gegner weg) |
| Pistole (Gun) | 1 | 125 | 10, Regeneration alle 500 ms | Projektil, Speed 2200, Kurve 1.25, Lebenszeit 2 s |
| Shotgun | 1 pro Kugel | 500 | 10 | Mehrere Kugeln mit Streuung, Speed 2750, SpeedDiff 0.8, Lebenszeit 0.2 s, Knockback |
| Granatwerfer | bis 6 (Explosion) | 500 | 10 | Ballistisch (Kurve 7.0), Speed 1000, Explosion mit Radius-Schaden und Knockback (Rocket-Jump) |
| Laser (Rifle) | 5 | 800 | 10 | Hitscan, Reichweite 800, prallt 1× an Wänden ab (150 ms Verzögerung) |
| Ninja | 9 | 800 | – | Power-Up (15 s), Dash-Angriff (200 ms, Velocity 50), ersetzt temporär andere Waffen |

- **Curvature** = Stärke der Flugbahn-Krümmung durch Gravitation.
- **Explosion:** Radius 135, innerer Radius 48 (voller Schaden), `MaxForce` 12. Faktor fällt linear von 1 (≤ 48) auf 0 (135). Schaden = `(int)(Faktor · MaxDamage)`, Kraft = `Richtung · MaxForce · Faktor`.
- **Hammer-Knockback:** `(0, -1) + normalize(Dir + (0, -1.1)) · 10` – schleudert immer leicht nach oben. Trifft nur bei freier Sichtlinie.
- **Eigenschaden:** `max(1, Dmg / 2)` – für alle Waffen, vor Rüstungsberechnung.
- **Pickups:** Respawn 15 s (Ninja 90 s, erstes Spawnen ebenfalls nach 90 s).
- **Velocity Ramp:** `1 / Curvature^((v - Start) / Range)`.
- Waffenwechsel per Mausrad / Zahlentasten; kurze Wechselzeit.

## 7. Gesundheit, Pickups, Tod

- **10 Herzen (HP)**, **10 Schilde (Rüstung)**. Rüstung absorbiert Schaden vor HP.
- Pickups: Herz (+1 HP), Schild (+1 Rüstung), Waffen (voll Munition), Ninja. Pickups respawnen nach fester Zeit.
- Eigenschaden durch eigene Granate (reduziert).
- Tod → kurzer Respawn-Delay (~0,5 s), Spawn an zufälligem/weitesten Spawnpunkt.
- Selbstmord (`kill`) möglich.

## 8. Spielmodi (Vanilla)

| Modus | Beschreibung |
|---|---|
| **DM** | Deathmatch, jeder gegen jeden |
| **TDM** | Team-Deathmatch, Rot vs. Blau |
| **CTF** | Capture the Flag – Flagge des Gegners zur eigenen Basis bringen |
| **LMS** (0.7) | Last Man Standing |
| **LTS** (0.7) | Last Team Standing |

Einstellbar: Score-Limit, Zeitlimit, Warmup, Team-Balance, Friendly Fire.
Community-Modi (nicht Vanilla): **DDRace** (kooperatives Parkour, Freeze-Tiles), **Instagib** (Varianten iDM/iTDM/iCTF: nur Laser, ein Treffer tötet; ob in 0.7 Vanilla enthalten, ist noch zu prüfen), zCatch, Race.

## 9. Steuerung (Default)

| Aktion | Taste |
|---|---|
| Laufen | A / D |
| Springen / Doppelsprung | Leertaste |
| Zielen | Maus (freie 360°-Zielrichtung) |
| Schießen | Linke Maustaste |
| Hook | Rechte Maustaste (halten) |
| Waffe wechseln | Mausrad / 1–5 |
| Emote | E (Rad) / Tastenkürzel |
| Chat / Team-Chat | T / Y |
| Scoreboard | Tab |
| Kill | K (konfigurierbar) |

Die Kamera folgt dem Spieler und verschiebt sich Richtung Mauszeiger (dynamische Kamera).

## 10. Netzwerk

- **Client-Server**, autoritativer Server, eigenes Protokoll über **UDP**.
- Server simuliert mit 50 TPS, sendet **Snapshots** (standardmäßig jeden 2. Tick → 25 Hz), **delta-komprimiert** gegen zuletzt bestätigten Snapshot.
- Client sendet Inputs (Richtung, Zielpunkt, Sprung, Feuer, Hook, Waffenwahl) pro Tick.
- **Client-Prediction** der eigenen Figur (und in DDNet auch anderer), Interpolation fremder Objekte.
- **Master-Server** für Server-Liste, Server-Browser im Client.
- Demo-Aufnahmen (Replays) über aufgezeichnete Snapshots.

## 11. Karten & Editor

- Eigenes Map-Format (`.map`, Datafile mit Gruppen/Layern).
- Layer-Typen: **Game-Layer** (Kollision), **Tile-Layer** (Grafik, Tilesets), **Quad-Layer** (freie Polygone, animierbar), **Entities**.
- Parallax-Gruppen für Hintergründe.
- Integrierter **Map-Editor** im Client.
- Bekannte Vanilla-Karten: `dm1`, `dm2`, `dm6`, `dm7`, `dm8`, `dm9`, `ctf1`–`ctf7`.

## 12. Grafik & Audio

- Handgezeichneter, runder Cartoon-Stil, kräftige Farben.
- Tee-Skins aus Teilen (Körper, Füße, Augen, Hände; 0.7 zusätzlich Deko/Markierungen), einfärbbar.
- Emotes über dem Kopf, Augen-Ausdrücke (Schmerz, Freude, …).
- Partikel: Rauch, Treffer, Explosionen, Blut-ähnliche „Splats“ in Skinfarbe.
- Kurze, markante Sounds für jede Waffe, Hook, Sprung, Treffer, Pickup, Tod.

## 13. Interface

- Hauptmenü: Server-Browser (Internet/LAN/Favoriten), Einstellungen (Spieler, Tee-Skin, Steuerung, Grafik, Sound), Demos, Editor.
- HUD: Herzen/Schilde, Munition, Waffenanzeige, Timer, Score, Killfeed, Chat, Emotes.
- Ingame-Konsole (lokal und Remote-Console für Admins).

## 14. Rechtliches (wichtig für einen Klon)

- Code-Lizenz erlaubt Weiterverwendung unter Bedingungen (Nennung, keine Falschdarstellung als Original).
- Assets (Grafik/Sound) stehen unter **CC-BY-SA 3.0** → Nutzung möglich mit Namensnennung und gleicher Lizenz.
- Name „Teeworlds“ sollte nicht als Produktname verwendet werden.
- → Wie wir damit umgehen, ist eine offene Entscheidung (siehe Entscheidungsdokument).
