# M7 – Menüs & Infrastruktur: Umsetzungsplan

Status: **Entwurf, Entscheidungen offen** · Grundlage: [`06-roadmap.md`](06-roadmap.md) M7, E-031, E-082, E-111, O-17, O-18, O-45

## Ziel

Ein neuer Spieler startet Elora, findet ohne Hilfe einen Server und kann spielen. Dazu kommen Hauptmenü, Einstellungen (Spieler und Skin, Steuerung, Grafik, Ton), Tastenbelegung und Server-Browser. Das Debug-Panel (egui) bleibt ein Entwicklerwerkzeug (E-031).

**Abnahme (Roadmap):** Ein neuer Spieler findet ohne Hilfe einen Server und kann spielen.

## Was das Original macht (Teeworlds 0.7, `menus*.cpp`, `serverbrowser.cpp`, `mastersrv`)

| Bereich | Original |
|---|---|
| **Start** | Client startet ins Hauptmenü; Hintergrund ist eine laufende Karte mit langsamer Kamerafahrt |
| **Hauptmenü** | Spielen (Server-Browser), Demos, Editor, Einstellungen, Beenden |
| **Server-Browser** | Reiter Internet / LAN / Favoriten; Liste mit Name, Spieltyp, Karte, Spieler/Max, Ping; Filter (leer, voll, Passwort, Spieltyp, Ping, Land); Detailansicht mit Spielerliste; Direkt-Verbinden per Adresse |
| **Master-Server** | Server melden sich regelmäßig per UDP beim Master an (Heartbeat mit Token), der Client holt dort die Adressliste und fragt **jeden Server selbst** nach Infos (verbindungslose Info-Pakete, daraus auch der Ping). DDNet ist später auf einen HTTP/JSON-Master umgestiegen |
| **LAN** | Broadcast der Info-Anfrage an die Ports 8303–8310 im lokalen Netz |
| **Einstellungen** | Spieler (Name, Clan, Land), Tee (Skin-Teile, Farben), Steuerung (Tastenbelegung, Maus-Empfindlichkeit), Grafik (Auflösung, Vollbild, VSync, FSAA, Texturqualität), Ton (Lautstärken, Musik), Allgemein |
| **Ingame-Menü (Esc)** | Spiel (Team wählen, Zuschauen, Trennen), Server-Info, Abstimmungen (Call Vote), Einstellungen |
| **Demos** | Client zeichnet Snapshots auf (`.demo`), Abspielen mit Zeitleiste, Pause, Tempo |
| **Musik** | Nur im Menü (E-082 übernimmt das) |

## Stand in Elora

- Der Client startet direkt in die **Sandbox**; Verbinden, Hosten (Server als eigener Prozess, `hosting.rs`), Team, Abstimmungen, Skin und Name laufen über das **Debug-Panel** (F1) bzw. `--connect`.
- **Name und Skin werden nicht gespeichert**; `tuning.toml` hält Tuning, Sicht, Maus, Effekte, Ton.
- **Netzwerk:** Verbindungsaufbau mit Token (Anfrage auf 512 Byte aufgefüllt), Noise-Verschlüsselung, TOFU-Serverschlüssel. **Es gibt noch keine verbindungslose Info-Abfrage** – die braucht der Server-Browser (Ping, Name, Karte, Spieler).
- **Spiel-UI:** Vektortext, Flächen, Symbole im HUD-Stil (M5.8); es gibt noch **keine Eingabe-Elemente** (Knöpfe, Schieberegler, Textfelder, Listen) – die entstehen hier als kleines eigenes UI-Toolkit.

## Arbeitsschritte (Vorschlag)

| # | Schritt | Crate | Inhalt | Prüfung |
|---|---|---|---|---|
| M7.1 | UI-Toolkit | Client (`ui/`) | Knopf, Schalter, Schieberegler, Textfeld, Liste mit Auswahl und Scrollen, Reiter, Tastatur-/Maus-Fokus; im HUD-Stil, skaliert mit der Fensterhöhe | Tests + Sichtprüfung |
| M7.2 | Einstellungen speichern | Client | Datei für Spieler, Skin, Steuerung, Grafik, Ton (D-M7-04) | Tests |
| M7.3 | Hauptmenü & Ablauf | Client | Startbildschirm, Menü-Zustände (Menü ↔ Spiel ↔ Ingame-Menü), Hintergrund (D-M7-02), Musik (D-M7-09) | Sichtprüfung |
| M7.4 | Einstellungsseiten | Client | Spieler & Skin (Vorschau der Figur), Steuerung, Grafik, Ton | Sichtprüfung |
| M7.5 | Tastenbelegung | Client | Aktionen statt fester Tasten, Neu-Belegen per Tastendruck, Konflikte anzeigen (D-M7-05) | Tests |
| M7.6 | Server-Info-Abfrage | `elora-net`, `elora-protocol`, Server | Verbindungslose Info-Anfrage mit Token gegen Verstärkungsangriffe; Antwort: Name, Karte, Modus, Spieler (Namen, Punkte), Max, Version, Passwort | Tests |
| M7.7 | Server-Browser | Client | LAN (Broadcast), Favoriten, Direkt-Verbinden, Internet über Master (D-M7-03); Liste, Sortieren, Filter, Details, Ping | Integrationstest + Sichtprüfung |
| M7.8 | Master-Server | `elora-master` (neu) je nach D-M7-03 | Anmeldung der Server, Liste für Clients, Ablauf nicht mehr gemeldeter Server | Tests |
| M7.9 | Ingame-Menü | Client | Esc: Fortsetzen, Team/Zuschauen, Abstimmung starten, Einstellungen, Trennen, Beenden – ersetzt die Spiel-Teile des Debug-Panels | Sichtprüfung |
| M7.10 | Demos (optional) | Client | je nach D-M7-08 | Tests |
| M7.11 | Abnahme | – | neuer Spieler ohne Hilfe im Spiel | Deine Abnahme |

## Entscheidungen zu M7

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-M7-01 | Menü-Design | 2–3 Entwürfe als Bild zur Auswahl (wie bei HUD und Figur) | offen |
| D-M7-02 | Start und Hintergrund | Hauptmenü mit laufender Karte im Hintergrund / ruhiges Standbild / direkt ins Training | offen |
| D-M7-03 | Internet-Serverliste (O-17) | eigener UDP-Master (wie Original) / HTTP/JSON-Master (wie DDNet) / für Release 1 nur LAN + Favoriten + Direkt | offen |
| D-M7-04 | Einstellungsdatei | eine neue `settings.toml` für alles (Tuning bleibt in `tuning.toml`) / alles in einer Datei | offen |
| D-M7-05 | Tastenbelegung | eine Taste je Aktion / zwei (Haupt + Neben) / beliebig viele | offen |
| D-M7-06 | Grafik-Einstellungen | Umfang: Fenster/Vollbild, VSync, MSAA, UI-Skalierung, FPS-Grenze … | offen |
| D-M7-07 | Server-Passwort | einführen (Browser zeigt Schloss) / nicht für Release 1 | offen |
| D-M7-08 | Demos (O-18) | in M7 / später (nach Release 1) / gar nicht | offen |
| D-M7-09 | Menümusik (E-082) | du lieferst die Musik (E-109) – ich baue Abspielen mit Schleife und Lautstärke | offen |
| D-M7-10 | Sprache der Oberfläche | nur Deutsch / Deutsch + Englisch umschaltbar / nur Englisch | offen |
| D-M7-11 | Remote-Konsole (O-45) | in M7 (Ingame-Menü, Passwort) / später | offen |
| D-M7-12 | Hosten aus dem Menü | „Server starten“ im Menü (vorhandenes `hosting.rs`) / nur Debug-Panel | offen |

## Technische Festlegungen (Vorschlag)

- **Eigenes UI-Toolkit im Sofortmodus** (immediate mode, wie egui, aber mit dem Vektor-Renderer und im Spielstil): jede Seite zeichnet sich pro Frame neu und liefert Aktionen zurück – passt zur bestehenden HUD-Zeichnung und braucht keine Zusatz-Bibliothek.
- **Info-Abfrage** als eigener, unverschlüsselter Pakettyp neben dem Handshake: kleine Anfrage mit Token → Antwort nur an die anfragende Adresse und höchstens so groß wie nötig; Rate-Grenze je Adresse. Die Server-Identität prüft weiterhin der Handshake (TOFU).
- **Zustände im Client:** `Menü` · `Spiel (Sandbox/Online)` · `Ingame-Menü` als klare Zustandsmaschine; Tasten gehen je nach Zustand an Menü oder Spiel.
