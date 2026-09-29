# M3 – Netzwerk: Umsetzungsplan

Status: **Vorschlag, wartet auf Entscheidung** · Grundlage: [`06-roadmap.md`](06-roadmap.md) M3, E-008, E-012, Analyse §10

## Ziel

Ein dedizierter Server, mehrere Clients über UDP. Die eigene Bewegung fühlt sich bei 100 ms Ping genauso an wie offline. Abnahme: 2 bis 8 Spieler im LAN, zusätzlich ein Test mit simuliertem Ping, Jitter und Paketverlust.

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M3.1 | Serialisierung | `elora-protocol` | Bit-/Byte-Packer mit variabler Ganzzahl-Länge, Nachrichten (Eingabe, Snapshot, Ereignisse, Verbindung), Versionsnummer | Unit-Tests: Rundlauf, fehlerhafte Pakete werden abgelehnt |
| M3.2 | Transport | `elora-net` | UDP-Socket, Verbindungsaufbau mit Token (gegen gefälschte Absender), Keepalive, Timeout, Trennen mit Grund; zuverlässige und unzuverlässige Kanäle (Sequenz, Ack, Resend), Aufteilen großer Nachrichten | Tests über eine simulierte Leitung mit Verlust und Umordnung |
| M3.3 | Netzwerk-Simulator | `elora-net` | Einstellbarer Ping, Jitter, Paketverlust, Umordnung, zwischen Socket und Transport geschaltet | Tests; Regler im Debug-Panel |
| M3.4 | Snapshots | `elora-protocol` | Vollständiger Welt-Snapshot (Figuren, Projektile, Laser, Pickups, Spielerinfos), **Delta gegen den zuletzt bestätigten Snapshot**, CRC | Tests: Delta-Rundlauf bit-genau, Größenmessung |
| M3.5 | Server | `apps/elora-server` | Dedizierter Server ohne Grafik: Karte laden, Spieler aufnehmen/entfernen, Eingaben pro Tick anwenden, Snapshots senden, Eingabe-Timing zurückmelden; Konfiguration per Datei/Kommandozeile | Headless-Integrationstest: Server + 2 Test-Clients |
| M3.6 | Client: Verbindung & Interpolation | `elora-client` | Verbinden per IP:Port, Snapshots empfangen, fremde Figuren und Objekte zwischen Snapshots interpolieren | Manuell im LAN |
| M3.7 | Client: Vorhersage | `elora-client` | Eingaben puffern und mit Ziel-Tick senden, Vorhersagezeit regeln (wie `INPUTTIMING`), vom Snapshot vorwärts rechnen, Korrektur bei Abweichung (siehe D-M3-03) | Test: Vorhersage = Server bei verlustfreier Leitung |
| M3.8 | Lokal hosten | `elora-client` | „Server starten“ aus dem Client (siehe D-M3-06), dann automatisch verbinden | Manuell |
| M3.9 | Debug & Messwerte | `elora-client` | Panel: Ping, Paketverlust, Bandbreite, Vorhersage-Ticks, Korrekturen; Netzwerk-Simulator-Regler | Sichtprüfung |
| M3.10 | Abnahme | – | 2–8 Spieler im LAN, Test mit 100 ms simuliertem Ping | Deine Abnahme |

## Technische Festlegungen (Vorschlag)

- **Keine Async-Laufzeit:** `std::net::UdpSocket` (nicht blockierend) im eigenen Netzwerk-Thread, Kanäle zur Spiellogik. Einfach, gut testbar und für ein 50-TPS-Spiel ausreichend.
- **Transport und Protokoll sind getrennt:** `elora-net` weiß nichts vom Spiel (Bytes rein, Bytes raus). `elora-protocol` kennt die Nachrichten und Snapshots, aber keine Sockets.
- **Transport testbar ohne echtes Netz:** Die Socket-Schicht ist austauschbar. Tests laufen über eine simulierte Leitung im Speicher.
- **Simulation bleibt unverändert:** Server und Client-Vorhersage benutzen denselben `World::step`. Für die Vorhersage bekommt `elora-sim` die Möglichkeit, einen Welt-Zustand aus einem Snapshot zu übernehmen.
- **Sandbox bleibt:** Ohne Verbindung läuft der Client weiter lokal wie bisher (für Tuning, Dummies und Aufzeichnungen).
- **Spielregeln:** Der Server spielt in M3 „frei für alle“ ohne Punkte, die Regeln folgen in M4.

## Entscheidungen zu M3

| # | Frage | Original-Verhalten (0.7) |
|---|---|---|
| D-M3-01 | Maximale Spieler pro Server | Standard 8, technisch bis 64 |
| D-M3-02 | Snapshot-Rate | 25 Hz (jeder 2. Tick), 50 Hz als LAN-Option |
| D-M3-03 | Umfang der Vorhersage | nur eigene Bewegung/Hook; andere interpoliert; Waffen nicht vorhergesagt |
| D-M3-04 | Lag-Kompensation für Treffer | keine |
| D-M3-05 | Verschlüsselung / Schutz | Token-Handshake, keine Verschlüsselung |
| D-M3-06 | Lokal hosten | Server als eigener Prozess, der Client startet ihn |
| D-M3-07 | Kompression | Huffman mit fester Tabelle + Delta |
