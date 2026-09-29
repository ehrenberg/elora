# M4 – Spielmodi: Umsetzungsplan

Status: **abgeschlossen** (E-092) · angenommen (E-066–E-079) · Grundlage: [`06-roadmap.md`](06-roadmap.md) M4, E-014, E-026, E-055, Analyse §8

## Ziel

DM, TDM, CTF, LMS, LTS und Instagib sind auf dem Server spielbar – mit Teams, Punkten, Runden, Siegbedingungen, Scoreboard, Killfeed und Chat. Abnahme: Jeder Modus in einer Playtest-Runde.

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M4.1 ✅ | Regel-Rahmen | `elora-game` (neu) | Spielzustände (Aufwärmen, Countdown, läuft, Rundenende, Match-Ende), Punkte, Score-/Zeitlimit, Sudden Death, Ereignisse (Kill, Runde, Sieg) | Unit-Tests mit Welt ohne Netz |
| M4.2 ✅ | Teams | `elora-sim` + `elora-game` | Team pro Spieler (Rot/Blau/Zuschauer), Friendly Fire (Schaden aus, Rückstoß bleibt), Team-Spawnpunkte, Team-Ausgleich, Teamtausch nach Match | Tests |
| M4.3 ✅ | DM / TDM | `elora-game` | Punkte wie Original, TDM-Respawn-Verzögerung | Tests |
| M4.4 ✅ | CTF | `elora-game` | Flaggen mit Physik (fallen, abprallen, 30 s-Rückkehr), Aufnehmen, Zurückbringen, Erobern | Tests |
| M4.5 ✅ | LMS / LTS | `elora-game` | kein Respawn in der Runde, Startausrüstung, Rundensieg | Tests |
| M4.6 ✅ | Instagib | `elora-game` | nur Laser, unendliche Munition, 1 Treffer = Tod, keine Pickups (E-026) | Tests |
| M4.7 ✅ | Server-Integration | `elora-server` | Modus und Regeln in `server.toml`/Kommandozeile, Spielzustand und Punkte im Snapshot, Kartenrotation | Integrationstest |
| M4.8 ✅ | Chat & Befehle | Protokoll, Server, Client | Chat und Team-Chat, `kill` (E-055), Team wählen/Zuschauer, Server-Konsole (D-M4-07) | Tests |
| M4.9 ✅ | Anzeige | Client | Scoreboard (Tab), Killfeed, Chat-Fenster, Rundenende-/Sieger-Anzeige, Timer, Flaggenanzeige – als Platzhalter (finales HUD in M5) | Sichtprüfung |
| M4.10 ✅ | Abnahme | – | Playtest aller Modi | Deine Abnahme |

## Technische Festlegungen (Vorschlag)

- **Neues Crate `elora-game`** (wie in der Architektur vorgesehen): Regeln oberhalb der Simulation. `elora-sim` bleibt regelfrei, bekommt aber das, was die Vorhersage braucht: Team-Zugehörigkeit (für Friendly Fire) und die Flaggen-Physik als Entity.
- **Regeln laufen nur auf dem Server** (und optional in der Sandbox, siehe D-M4-10). Der Client bekommt Spielzustand, Punkte, Teams und Flaggen über den Snapshot und Nachrichten (Chat, Killfeed).
- **Die Sandbox bleibt ohne Regeln lauffähig**, damit Tuning und Aufzeichnungen unverändert funktionieren.

## Entscheidungen zu M4

| # | Frage | Original-Verhalten (0.7) |
|---|---|---|
| D-M4-01 → E-066 | Siegbedingungen (Standard je Modus) | Score-Limit 20, Zeitlimit aus, Sudden Death bei Gleichstand |
| D-M4-02 → E-067 | CTF-Wertung | Eroberung = Teampunkt; Träger +5, Aufnehmen/Zurückbringen/Träger töten +1 |
| D-M4-03 → E-068 | Aufwärmen / Countdown | Aufwärmen 0 s (unbegrenzt bei zu wenigen Spielern), Countdown aus, Survival 3 s |
| D-M4-04 → E-069 | Friendly Fire | aus (Rückstoß wirkt trotzdem) |
| D-M4-05 → E-070 | Respawn-Verzögerung TDM | 3 s |
| D-M4-06 → E-071 | Startausrüstung LMS/LTS | +5 Rüstung, Shotgun, Granate 10, Laser 5 |
| D-M4-07 → E-072 | Konsole / Admin (O-20) | lokale Server-Konsole + Remote-Konsole mit Passwort |
| D-M4-08 → E-073 | Team-Wahl und Zuschauer | Spieler wählen Team oder Zuschauer; automatischer Ausgleich |
| D-M4-09 → E-074 | Kartenrotation und Teamtausch | Rotation per Liste, Teamtausch nach jedem Match |
| D-M4-10 → E-075 | Spielmodi auch in der Sandbox (gegen Dummies)? | – |
| D-M4-11 → E-076 | Instagib: für welche Modi? | Community: iDM, iTDM, iCTF |
| D-M4-12 → E-078 | Chat-Tasten | T allgemein, Y Team |

## Ausarbeitung

- **Aufwärmen (E-068):** Solange zu wenige Spieler da sind (DM < 2, Teammodi: ein Team leer), bleibt das Spiel wie im Original im unbegrenzten Aufwärmen. Nach einem Kartenwechsel 10 s Aufwärmen, danach 3 s Countdown; nach Match-Ende (10 s) bzw. Rundenende (5 s) direkt 3 s Countdown.
- **Friendly Fire (E-069):** Standard „an“ als Server-Einstellung `friendly_fire`; bei „aus“ verhält es sich wie im Original (kein Schaden, Rückstoß bleibt).
- **Abstimmungen (E-077):** Dauer 25 s; angenommen sofort, wenn mehr als die Hälfte der Spieler Ja stimmt, abgelehnt sofort bei mindestens der Hälfte Nein, sonst nach Ablauf angenommen bei mehr Ja als Nein. Kick sperrt die Adresse 5 min. Eine Abstimmung gleichzeitig.
- **Instagib (E-076):** Spawn nur mit Laser (unbegrenzte Munition), ein Treffer tötet, keine Pickups.
- **Sandbox (E-075):** Modus im Panel wählbar; Aufzeichnungen (F5) nur ohne Modus, damit Golden-Tests reine Simulation bleiben.

## Umsetzungsnotizen (Stand 2026-09-29)

- **Testkarte:** `maps/ctf-test.emap.toml` mit Team-Spawns, Flaggen, Pickups und Dummies. Sandbox: `cargo run --bin elora -- maps/ctf-test.emap.toml --mode ctf`; Server: `cargo run --bin elora-server -- --map maps/ctf-test.emap.toml --mode ctf`.
- **Dummies zählen als Spieler** (Teams, „genug Spieler“, Punkte) – auf Test-Servern praktisch, auf echten Karten gibt es keine Dummies.
- **Anzeigen (Platzhalter bis M5):** Statusleiste (Modus, Timer, Phase, Teampunkte bzw. eigene Punkte, Ziel, Sudden Death), Abstimmungsbanner, Killfeed, Chat mit Hinweisen, Scoreboard (Tab), Teamfarben (eigene Figur mit gelbem Ring), Flaggen mit Stand-Markierung.
- **Konsole und Abstimmungen:** siehe README.
- **Nicht per Maus durchgeklickt:** Chat-Eingabe, Abstimmungs-Dialog und Team-Buttons im Fenster (die Logik dahinter ist durch Integrationstests abgedeckt).
- **Huffman-Tabelle** wurde noch auf dem Snapshot-Format vor M4 trainiert; sie funktioniert weiter (bei Bedarf wird unkomprimiert gesendet). Neu trainieren mit `cargo xtask train-huffman`, sobald echte Spieldaten vorliegen.

