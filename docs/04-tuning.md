# Tuning-Vorschlag für Elora

Status: **angenommen** (E-023, 2026-09-25) – alle Werte T-01 bis T-30 gelten als Startwerte; T-31 bis T-36 seit E-140.

Original-Werte stammen aus [`01-analyse-teeworlds.md`](01-analyse-teeworlds.md). Einheiten: 1 Tile = 32 Einheiten, Werte gelten pro Tick.

## Leitidee

E-015 verlangt eine Abweichung, aber nur „etwas“. Deshalb gilt:

- **Die meisten Werte ändern sich um höchstens ±5–10 %.** Größere Änderungen wirken sofort fremd.
- **Die Kernmechaniken bleiben unangetastet** (Luftreibung, Velocity Ramp, Bodenreibung). Sie prägen das Gefühl am stärksten.
- **Elora bekommt eine eigene Charakteristik:** etwas **flinker am Boden**, ein **stärkerer erster Sprung**, dafür ein **schwächerer Doppelsprung**, und ein **etwas längerer, schnellerer Hook** mit **weniger Zug**. Das Ergebnis: mehr Tempo und Schwung, ohne dass die Gesamt-Sprunghöhe steigt.
- Alle Werte kommen in `elora-sim/src/tuning.rs` und lassen sich in der Sandbox live ändern. Die Endwerte legen wir nach dem Anspielen fest.

## A. Simulationsrate

| # | Wert | Original | Vorschlag | Begründung | Entscheidung |
|---|---|---|---|---|---|
| T-01 | Ticks pro Sekunde | 50 | **50** (unverändert) | Alle Werte sind pro Tick definiert. Bei einer anderen Rate müssten wir alles umrechnen, und das Gefühl verschiebt sich unkontrolliert. | ✅ |

## B. Bewegung

| # | Wert | Original | Vorschlag | Δ | Begründung | Entscheidung |
|---|---|---|---|---|---|---|
| T-02 | GroundControlSpeed | 10.0 | **10.5** | +5 % | 525 statt 500 Einheiten/s (≈ 16,4 Tiles/s). Elora wirkt etwas flinker. | ✅ |
| T-03 | GroundControlAccel | 2.0 | **2.2** | +10 % | Elora erreicht die Maximalgeschwindigkeit schneller, die Steuerung reagiert direkter. | ✅ |
| T-04 | GroundFriction | 0.5 | **0.5** | – | Kernwert für das Abbremsen, bleibt. | ✅ |
| T-05 | GroundJumpImpulse | 13.2 | **13.6** | +3 % | Sprunghöhe ≈ 185 statt 174 Einheiten (5,8 statt 5,4 Tiles). | ✅ |
| T-06 | AirJumpImpulse | 12.0 | **11.5** | −4 % | Doppelsprung ≈ 132 statt 144 Einheiten. **Die Gesamthöhe bleibt fast gleich** (≈ 317 statt 318), der erste Sprung zählt aber mehr. | ✅ |
| T-07 | AirControlSpeed | 5.0 | **5.0** | – | Luftkontrolle ist Kern-Feeling. | ✅ |
| T-08 | AirControlAccel | 1.5 | **1.6** | +7 % | Minimal mehr Lenkbarkeit in der Luft | ✅ |
| T-09 | AirFriction | 0.95 | **0.95** | – | Kernwert, bleibt | ✅ |
| T-10 | Gravity | 0.5 | **0.5** | – | Hängt an allen Flugbahnen. Die Sprunghöhe regeln wir stattdessen über die Impulse. | ✅ |
| T-11 | VelrampStart / Range / Curvature | 550 / 2000 / 1.4 | **unverändert** | – | Die Obergrenze für die Geschwindigkeit bleibt. | ✅ |

## C. Hook

| # | Wert | Original | Vorschlag | Δ | Begründung | Entscheidung |
|---|---|---|---|---|---|---|
| T-12 | HookLength | 380 | **400** | +5 % | 12,5 statt 11,9 Tiles. Elora greift minimal weiter. | ✅ |
| T-13 | HookFireSpeed | 80 | **85** | +6 % | Gleicht T-12 aus: Trotz größerer Reichweite braucht der Hook bis zur maximalen Länge so lange wie im Original (≈ 4,7 statt 4,75 Ticks). Auf kurze Distanz trifft er etwas schneller. | ✅ |
| T-14 | HookDragAccel | 3.0 | **3.0** | – | Bleibt | ✅ |
| T-15 | HookDragSpeed | 15.0 | **14.0** | −7 % | Gleicht die größere Reichweite aus, der Hook bleibt Werkzeug statt Katapult. | ✅ |
| T-16 | Player-Hook-Dauer | 60 Ticks (1,2 s) | **55 Ticks (1,1 s)** | −8 % | Eine etwas kürzere Kontrolle über Gegner macht das Spiel weniger frustrierend. | ✅ |
| T-17 | Player-Hook-Kraftfaktor | 1.5 | **1.5** | – | Bleibt | ✅ |

> **Hinweis zu T-12/T-13 (Erkenntnis aus M1.1):** Die *effektive* Wand-Reichweite ergibt sich aus den Flugschritten (siehe Analyse §5): Elora **382** (42 + 4·85), Original **362** (42 + 4·80) – also +5,5 %. Spieler werden bis zur vollen Länge (400) getroffen.

## D. Waffen (E-016: Hammer, Laser, Granate)

| # | Wert | Original | Vorschlag | Begründung | Entscheidung |
|---|---|---|---|---|---|
| T-18 | Hammer Schaden / Feuerrate | 3 / 125 ms | **3 / 125 ms** | Ohne Pistole ist der Hammer die einzige Grundwaffe. Er sollte verlässlich bleiben. | ✅ |
| T-19 | Hammer Knockback | 10 (+1,1 Aufwärtsanteil) | **11** | Etwas stärker, der Hammer bekommt mehr Charakter als Mobilitätswerkzeug. | ✅ |
| T-20 | Laser Schaden / Feuerrate | 5 / 800 ms | **5 / 750 ms** | Etwas höhere Kadenz als Ausgleich dafür, dass Pistole und Shotgun fehlen | ✅ |
| T-21 | Laser Reichweite | 800 | **850** | Leicht größer, passend zum längeren Hook | ✅ |
| T-22 | Laser Abpraller / Verzögerung | 1 / 150 ms | **1 / 150 ms** | Bleibt | ✅ |
| T-23 | Granate Schaden / Feuerrate | 6 / 500 ms | **6 / 500 ms** | Bleibt, Balance-Anker | ✅ |
| T-24 | Granate Speed / Curvature / Lifetime | 1000 / 7.0 / 2 s | **1050 / 7.0 / 2 s** | Etwas schneller, direktere Flugbahn | ✅ |
| T-25 | Explosion Radius / Innen / MaxForce | 135 / 48 / 12 | **135 / 48 / 12.5** | Rocket-Jumps werden minimal stärker, das passt zu Eloras schwächerem Doppelsprung. | ✅ |
| T-26 | Eigenschaden | max(1, Dmg/2) | **max(1, Dmg/2)** | Bleibt | ✅ |
| T-27 | Max. Munition | 10 | **10** | Bleibt | ✅ |

## E. Leben & Pickups

| # | Wert | Original | Vorschlag | Begründung | Entscheidung |
|---|---|---|---|---|---|
| T-28 | Max. HP / Rüstung | 10 / 10 | **10 / 10** | Bestimmt die Time-to-Kill, das ist Kern-Feeling. | ✅ |
| T-29 | Pickup-Respawn | 15 s | **15 s** | Bleibt | ✅ |
| T-30 | Respawn-Verzögerung nach Tod | ≈ 0,5 s | **0,5 s** | Bleibt, schnelles Spiel | ✅ |

## E2. Tile-Arten (M6.1, E-137, E-140)

| # | Wert | Original | Vorschlag | Begründung | Entscheidung |
|---|---|---|---|---|---|
| T-31 | Bodenreibung auf Eis | – (Boden 0,5) | **0,985** | Figur rutscht nach dem Loslassen weit nach | ✅ |
| T-32 | Beschleunigung auf Eis | – (Boden 2,0) | **0,35** | Anlaufen und Bremsen dauern spürbar länger | ✅ |
| T-33 | Kraft Sprungfeld | – | **20** Einheiten/Tick | Wirft etwa 12 Tiles hoch, klar mehr als ein Sprung | ✅ |
| T-34 | Richtungen Sprungfeld | – | **hoch, schräg links, schräg rechts (45°)** | Reicht für Release 1 | ✅ |
| T-35 | Geschwindigkeit Beschleuniger | – | **4,0** Einheiten/Tick | Trägt spürbar, man kommt noch dagegen an | ✅ |
| T-36 | Plattform | – | **von unten/seitlich durchlässig**; Hook, Granate, Laser fliegen hindurch | Wie Einbahn-Plattformen in anderen Spielen | ✅ |

## F. Regeln aus Folgeentscheidungen

- **Startausrüstung (E-025):** Spawn nur mit Hammer. Laser und Granate gibt es per Pickup mit voller Munition (10).
- **Instagib (E-026):** nur Laser, Munition unendlich, 1 Treffer = Tod, keine Pickups.
