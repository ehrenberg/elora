# M3 – Netzwerk: Umsetzungsplan

Status: **abgeschlossen** (E-065) · angenommen (E-057–E-064) · Grundlage: [`06-roadmap.md`](06-roadmap.md) M3, E-008, E-012, Analyse §10

## Ziel

Ein dedizierter Server, mehrere Clients über UDP. Die eigene Bewegung fühlt sich bei 100 ms Ping genauso an wie offline. Abnahme: 2 bis 8 Spieler im LAN, zusätzlich ein Test mit simuliertem Ping, Jitter und Paketverlust.

## Arbeitsschritte

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M3.1 ✅ | Serialisierung | `elora-protocol` | Bit-/Byte-Packer mit variabler Ganzzahl-Länge, Nachrichten (Eingabe, Snapshot, Ereignisse, Verbindung), Versionsnummer | Unit-Tests: Rundlauf, fehlerhafte Pakete werden abgelehnt |
| M3.2 ✅ | Transport | `elora-net` | UDP-Socket, Verbindungsaufbau mit Token (gegen gefälschte Absender), Keepalive, Timeout, Trennen mit Grund; zuverlässige und unzuverlässige Kanäle (Sequenz, Ack, Resend), Aufteilen großer Nachrichten | Tests über eine simulierte Leitung mit Verlust und Umordnung |
| M3.3 ✅ | Netzwerk-Simulator | `elora-net` | Einstellbarer Ping, Jitter, Paketverlust, Umordnung, zwischen Socket und Transport geschaltet | Tests; Regler im Debug-Panel |
| M3.4 ✅ | Snapshots | `elora-protocol` | Vollständiger Welt-Snapshot (Figuren, Projektile, Laser, Pickups, Spielerinfos), **Delta gegen den zuletzt bestätigten Snapshot**, CRC | Tests: Delta-Rundlauf bit-genau, Größenmessung |
| M3.5 ✅ | Server | `apps/elora-server` | Dedizierter Server ohne Grafik: Karte laden, Spieler aufnehmen/entfernen, Eingaben pro Tick anwenden, Snapshots senden, Eingabe-Timing zurückmelden; Konfiguration per Datei/Kommandozeile | Headless-Integrationstest: Server + 2 Test-Clients |
| M3.6 ✅ | Client: Verbindung & Interpolation | `elora-client` | Verbinden per IP:Port, Snapshots empfangen, fremde Figuren und Objekte zwischen Snapshots interpolieren | Manuell im LAN |
| M3.7 ✅ | Client: Vorhersage | `elora-client` | Eingaben puffern und mit Ziel-Tick senden, Vorhersagezeit regeln (wie `INPUTTIMING`), vom Snapshot vorwärts rechnen, Korrektur bei Abweichung (siehe D-M3-03) | Test: Vorhersage = Server bei verlustfreier Leitung |
| M3.8 ✅ | Lokal hosten | `elora-client` | „Server starten“ aus dem Client (siehe D-M3-06), dann automatisch verbinden | Manuell |
| M3.9 ✅ | Debug & Messwerte | `elora-client` | Panel: Ping, Paketverlust, Bandbreite, Vorhersage-Ticks, Korrekturen; Netzwerk-Simulator-Regler | Sichtprüfung |
| M3.10 ✅ | Abnahme | – | 2–8 Spieler im LAN, Test mit 100 ms simuliertem Ping | Deine Abnahme |

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
| D-M3-01 → E-059 | Maximale Spieler pro Server | Standard 8, technisch bis 64 |
| D-M3-02 → E-059 | Snapshot-Rate | 25 Hz (jeder 2. Tick), 50 Hz als LAN-Option |
| D-M3-03 → E-057 | Umfang der Vorhersage | nur eigene Bewegung/Hook; andere interpoliert; Waffen nicht vorhergesagt |
| D-M3-04 → E-058 | Lag-Kompensation für Treffer | keine |
| D-M3-05 → E-061/E-062 | Verschlüsselung / Schutz | Token-Handshake, keine Verschlüsselung |
| D-M3-06 → E-060 | Lokal hosten | Server als eigener Prozess, der Client startet ihn |
| D-M3-07 → E-063 | Kompression | Huffman mit fester Tabelle + Delta |

## Ausarbeitung der Entscheidungen

### Vorhersage der eigenen Waffen (E-057)

Der Client baut aus jedem Snapshot eine lokale Welt und rechnet sie mit den eigenen, noch nicht bestätigten Eingaben bis zum Vorhersage-Tick vorwärts – mit demselben `World::step` wie der Server. In dieser **Vorhersage-Welt**:
- wirken Kräfte (Knockback, Rocket-Jump), aber **kein Schaden, kein Tod, keine Pickups** – das entscheidet allein der Server;
- andere Figuren bekommen **keine Eingaben** (wie `Tick(false)` im Original: Laufrichtung bleibt, kein neuer Sprung/Hook);
- eigene Projektile und Laserstrahlen werden aus der Vorhersage gezeichnet, fremde aus den interpolierten Snapshots.

### Verschlüsselung (E-061, E-062)

- Ablauf: Token-Anfrage (auf 512 Byte aufgefüllt, gegen Verstärkungsangriffe) → Token (an Absenderadresse gebunden) → Noise-Handshake `Noise_XX_25519_ChaChaPoly_BLAKE2s` → verschlüsselte Pakete mit expliziter Paketnummer als Nonce (UDP-tauglich, Wiedereinspiel-Schutz über Fenster).
- Der Server hat einen dauerhaften Schlüssel (Datei neben der Server-Konfiguration). Der Client speichert bekannte Server-Schlüssel (`known_servers.toml`) und warnt bei Änderung.

### Kompression (E-063)

Das Original bildet pro Objekt die Differenz aller Ganzzahlen zum Vorgänger, schreibt sie mit variabler Länge und komprimiert das Paket mit Huffman (feste Tabelle). Besser ist:
1. **Feldweises Delta mit Änderungsmaske:** Pro Objekt ein Bit pro Feld „geändert?“; unveränderte Felder kosten 1 Bit statt ≥ 1 Byte. Die meisten Felder (Leben, Waffe, Hook-Zustand) ändern sich selten.
2. **Kompakte Zahlen:** geänderte Felder als Differenz, ZigZag + variable Länge.
3. **Statischer Huffman** darüber, mit einer Tabelle, die auf **unserem** Verkehr trainiert ist (Werkzeug in `xtask`), statt der Tabelle des Originals.

Stufe 1 + 2 übertreffen erfahrungsgemäß „Delta + Huffman“ des Originals bereits; Stufe 3 holt den Rest. Gemessen und dokumentiert wird in M3.9 (Bytes pro Snapshot bei 8/16/64 Spielern).

## Umsetzungsnotizen (Stand 2026-09-29)

### Bedienung

- **Server starten:** `cargo run --bin elora-server -- [--port 8303] [--map maps/sandbox.emap.toml] [--max-clients 8] [--high-bandwidth] [--name "…"]` oder mit `--config server.toml`. Beenden: `quit` eingeben oder Strg+C.
- **Verbinden:** im Panel unter *Netzwerk* Adresse eintragen → *Verbinden*, oder `cargo run --bin elora -- --connect 127.0.0.1:8303`. *Trennen* führt zurück in die Sandbox.
- **Lokal hosten (E-060):** *Server einrichten …* → Name, Port, Karte, max. Spieler, 50 Hz, „weiterlaufen lassen“ → *Starten und verbinden*. Die Einstellungen landen in `server.toml`; der Client startet `elora-server` als eigenen Prozess.
- **TOFU (E-062):** Beim ersten Verbinden speichert der Client den Server-Schlüssel in `known_servers.toml`. Ändert er sich, erscheint eine Warnung mit beiden Fingerabdrücken und der Wahl „Neuem Schlüssel vertrauen“.
- **Simulator (M3.3/M3.9):** Im Online-Panel Verzögerung, Jitter und Verlust für ausgehende Pakete einstellen.

### Messwerte

Vorhersage im Integrationstest (Server + Client im simulierten Netz, 100 ms Ping):

| Größe | Wert |
|---|---|
| Vorhergesagte Ticks | 7 |
| Vorlauf der Eingaben | ≈ 125 ms |
| Restzeit der Eingaben beim Server | ≈ 15–25 ms (Ziel: > 10 ms) |
| Abweichung Vorhersage ↔ Server (verlustfrei) | **0** Einheiten |
| Abweichung bei 10 % Verlust + 30 ms Jitter | in 1 von 40 Stichproben > 0 (redundante Eingaben fangen fast alles ab) |

Größen (`cargo xtask net-stats`, synthetischer Verkehr, Delta gegen 3 Snapshots ältere Basis, alle Spieler laufen/springen/hooken/schießen):

| Spieler | Snapshot wie Original (Delta aller Felder + Huffman) | Elora roh | Elora + Huffman | vs. Original | Eingabe roh | Eingabe + Huffman | Server→Client pro Client (25 Hz) |
|---|---|---|---|---|---|---|---|
| 8 | 198 B | 209 B | 188 B | −5 % | 48 B | 31 B | 5,7 kB/s |
| 16 | 341 B | 357 B | 322 B | −5 % | 48 B | 31 B | 9,1 kB/s |
| 64 | 1169 B | 1224 B | 1106 B | −5 % | 48 B | 31 B | 28,6 kB/s |

**Einordnung zu E-063:** Das feldweise Delta mit Änderungsmaske plus trainiertem Huffman ist **messbar, aber nur leicht (≈ 5 %) besser** als das Verfahren des Originals mit derselben Tabelle – Huffman macht die vielen Null-Differenzen des Originals bereits billig, und bei vielen aktiven Spielern ändert sich fast jedes Objekt in jedem Snapshot. Deutlich mehr bringen würde ein Delta gegen eine **vorausberechnete** Basis (Position + Geschwindigkeit fortgeschrieben); das ist als mögliche Optimierung vermerkt, für Release 1 aber nicht nötig: selbst 64 Spieler brauchen < 30 kB/s pro Client. Echte Spiele im Sandbox-Leerlauf liegen bei ≈ 30–50 B pro Snapshot.

### Grenzen und offene Punkte

- **Waffen-Vorhersage (E-057):** eigene Schüsse, Laserstrahlen, Hammer-Effekte und Rückstoß erscheinen sofort; Schaden, Tod und Pickups kommen vom Server.
- **Einrichten-Dialog und Schlüssel-Warnung** sind gebaut, aber von mir nicht per Maus durchgeklickt (nur Start per `--connect` und der Server-Start per Kommandozeile wurden live getestet).
- **Server-Regeln:** frei für alle ohne Punkte (M4).
- **Karte:** der Server überträgt das Textformat; das Release-Format folgt in M6.

