# M7 – Menüs & Infrastruktur: Umsetzungsplan

Status: **angenommen** (E-112–E-127), in Umsetzung · Grundlage: [`06-roadmap.md`](06-roadmap.md) M7, E-031, E-082, E-111, O-17, O-18, O-45

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
| M7.0 ✅ | Entwürfe | – | 2–3 Menü-Entwürfe als Bild (E-123) | Deine Auswahl |
| M7.1 ✅ | UI-Toolkit | Client (`ui/`) | Knopf, Schalter, Schieberegler, Textfeld, Liste mit Auswahl und Scrollen, Reiter, Tastatur-/Maus-Fokus; im HUD-Stil, skaliert mit der Fensterhöhe | Tests + Sichtprüfung |
| M7.2 ✅ | Einstellungen & Sprache | Client | `settings.toml` im Benutzerverzeichnis (E-116): Spieler, Skin, Steuerung, Grafik, Ton, Sprache, Favoriten; Übersetzungsdateien Deutsch/Englisch (E-114) | Tests |
| M7.3 ✅ | Hauptmenü & Ablauf | Client | Hauptmenü über ruhigem Bild (E-113), Zustände Menü ↔ Spiel ↔ Ingame-Menü, Menümusik vorbereitet (E-121), „Server erstellen“ (E-122), Training (Sandbox) | Sichtprüfung |
| M7.4 ✅ | Einstellungsseiten | Client | Spieler & Skin (Vorschau der Figur), Steuerung, Grafik (E-120: Fenster/Vollbild, VSync, MSAA, UI-Skalierung), Ton, Sprache | Sichtprüfung |
| M7.5 ✅ | Tastenbelegung | Client | Aktionen statt fester Tasten, eine Taste je Aktion (E-117), Neu-Belegen per Tastendruck, Konflikte anzeigen | Tests |
| M7.6 ✅ | Server-Info-Abfrage | `elora-net`, `elora-protocol`, Server | Verbindungslose Info-Anfrage mit Token gegen Verstärkungsangriffe; Antwort: Name, Karte, Modus, Spieler (Namen, Punkte), Max, Version | Tests |
| M7.7 ✅ | Server-Browser | Client | Internet (Master, E-112), LAN (Broadcast), Favoriten, Direkt-Verbinden; Liste, Sortieren, Filter, Details mit Spielerliste, Ping | Integrationstest + Sichtprüfung |
| M7.8 ✅ | Master-Server | `elora-master` (neu) | HTTP/JSON (E-112): Server melden sich regelmäßig an (Adresse, Port; Master prüft Erreichbarkeit per Info-Abfrage), `GET` liefert die Liste als JSON, nicht mehr gemeldete Server laufen ab | Tests |
| M7.9 ✅ | Ingame-Menü | Client | Esc: Fortsetzen, Team/Zuschauen, Abstimmung starten, Einstellungen, Trennen, Beenden – ersetzt die Spiel-Teile des Debug-Panels | Sichtprüfung |
| M7.10 | Abnahme | – | neuer Spieler ohne Hilfe im Spiel | Deine Abnahme |

## Entscheidungen zu M7

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-M7-01 | Menü-Design | 2–3 Entwürfe als Bild zur Auswahl (wie bei HUD und Figur) | E-123: 2–3 Entwürfe als Bild (Hauptmenü, Browser, Einstellungen) |
| D-M7-02 | Start und Hintergrund | Hauptmenü mit laufender Karte im Hintergrund / ruhiges Standbild / direkt ins Training | E-113: Hauptmenü über ruhigem, gezeichnetem Bild |
| D-M7-03 | Internet-Serverliste (O-17) | eigener UDP-Master (wie Original) / HTTP/JSON-Master (wie DDNet) / für Release 1 nur LAN + Favoriten + Direkt | E-112: HTTP/JSON-Master (wie DDNet); Betrieb offen (O-47) |
| D-M7-04 | Einstellungsdatei | eine neue `settings.toml` für alles (Tuning bleibt in `tuning.toml`) / alles in einer Datei | E-116: `settings.toml` im Benutzerverzeichnis |
| D-M7-05 | Tastenbelegung | eine Taste je Aktion / zwei (Haupt + Neben) / beliebig viele | E-117: eine Taste je Aktion |
| D-M7-06 | Grafik-Einstellungen | Umfang: Fenster/Vollbild, VSync, MSAA, UI-Skalierung, FPS-Grenze … | E-120: Fenster/Vollbild, VSync, MSAA, UI-Skalierung |
| D-M7-07 | Server-Passwort | einführen (Browser zeigt Schloss) / nicht für Release 1 | E-118: kein Passwort für Release 1 |
| D-M7-08 | Demos (O-18) | in M7 / später (nach Release 1) / gar nicht | E-115: später |
| D-M7-09 | Menümusik (E-082) | du lieferst die Musik (E-109) – ich baue Abspielen mit Schleife und Lautstärke | E-121: Abspielen vorbereiten (`assets/music/`, Schleife, eigene Lautstärke) |
| D-M7-10 | Sprache der Oberfläche | nur Deutsch / Deutsch + Englisch umschaltbar / nur Englisch | E-114: Deutsch + Englisch, umschaltbar |
| D-M7-11 | Remote-Konsole (O-45) | in M7 (Ingame-Menü, Passwort) / später | E-119: später |
| D-M7-12 | Hosten aus dem Menü | „Server starten“ im Menü (vorhandenes `hosting.rs`) / nur Debug-Panel | E-122: „Server erstellen“ im Menü |

## Technische Festlegungen (Vorschlag)

- **Eigenes UI-Toolkit im Sofortmodus** (immediate mode, wie egui, aber mit dem Vektor-Renderer und im Spielstil): jede Seite zeichnet sich pro Frame neu und liefert Aktionen zurück – passt zur bestehenden HUD-Zeichnung und braucht keine Zusatz-Bibliothek.
- **Info-Abfrage** als eigener, unverschlüsselter Pakettyp neben dem Handshake: kleine Anfrage mit Token → Antwort nur an die anfragende Adresse und höchstens so groß wie nötig; Rate-Grenze je Adresse. Die Server-Identität prüft weiterhin der Handshake (TOFU).
- **Zustände im Client:** `Menü` · `Spiel (Sandbox/Online)` · `Ingame-Menü` als klare Zustandsmaschine; Tasten gehen je nach Zustand an Menü oder Spiel.

## Neue Abhängigkeiten (Vorschlag, Lizenzprüfung per cargo-deny)

- **Benutzerverzeichnis:** `directories` (MIT/Apache-2.0) für den Ort von `settings.toml` je Betriebssystem.
- **HTTP (E-112):** Client und Server als HTTP-Client (z. B. `ureq`, MIT/Apache-2.0, ohne TLS-Pflicht im LAN; für den öffentlichen Master TLS über `rustls`); Master als kleiner HTTP-Dienst (z. B. `tiny_http`, MIT/Apache-2.0). Endgültige Wahl nach Lizenz- und Abhängigkeitsprüfung.
- **Vollbild/VSync/MSAA:** vorhanden (winit, wgpu).

## Umsetzungsstand

- **M7.0 Entwürfe:** `docs/design/elora-menue.png` (A/B/C); gewählt (E-125): Aufbau von C in Farben und Formen von B → `docs/design/elora-menue-gewaehlt.png`. Generator `tools/design/elora_menu.py`.
- **M7.1 UI-Toolkit:** `apps/elora-client/src/ui.rs`, Sofortmodus mit Kennungen je Widget; Zustand (Fokus, gedrücktes Widget, Scroll) in `UiState`, Eingaben je Frame in `UiInput`. Widgets: Karte, Beschriftung, Pillen-Knopf (Schatten, Umriss, Hover heller, gedrückt rutscht er auf den Schatten), Reiter (waagerecht und als Seitenleiste), Schalter, Schieberegler, Textfeld (Fokus, Cursor, Umlaute, Rücktaste/Entf/Pfeile/Pos1/Ende, Enter, Esc, Höchstlänge), Farbfelder, Liste mit Spalten, Auswahl, Doppelklick, Mausrad und Laufleiste. Thema nach E-125. Galerie `docs/design/elora-ui-toolkit.png`.
- **M7.2 Einstellungen & Sprache:** `settings.rs` – `settings.toml` im Benutzerverzeichnis (Ort ohne Zusatz-Bibliothek bestimmt: `directories` hätte über `option-ext` MPL-2.0 mitgebracht), Speichern über Zwischendatei, beim Beenden; enthält Sprache, Name, Skin, Grafik (Vollbild, VSync, MSAA, UI-Skalierung 0.5–2), Ton, Effekte, Maus, Favoriten, letzter Server. `tuning.toml` (jetzt `tuning_file.rs`) nur noch Physik und Sicht. Renderer: `set_vsync`, `set_msaa` zur Laufzeit. `lang.rs` + `assets/lang/{de,en}.toml` (Abschnitte, Platzhalter `{name}`, Rückfall Deutsch → Schlüssel; Test: gleiche Schlüssel und Platzhalter); Startsprache nach Systemsprache. HUD und Spielanzeigen übersetzt. **Nicht übersetzt:** Meldungen, die der Server als fertigen Text schickt (Beitritt, Abstimmungen, Rundenende) → O-48.
- **M7.3 Hauptmenü & Ablauf:** `menu.rs` (Seiten, Pause-Menü) und `app_menu.rs` (Anbindung an die App). Zustand `Screen::Menu` / `Screen::Game`, darüber das Pause-Menü (Esc oder Fokusverlust; Fortsetzen, Zum Hauptmenü, Beenden – M7.9 erweitert es). Start im Hauptmenü; mit Karte, `--mode` oder `--connect` auf der Kommandozeile direkt ins Spiel (Entwicklung, dann auch mit Debug-Panel). Leiste oben: Spielen, Training (startet die Sandbox), Server erstellen, Einstellungen, Beenden. „Spielen“: Begrüßung, „Schnell spielen“ (letzter Server), Direkt-Verbinden mit Favoriten (Klick übernimmt, Doppelklick verbindet, merken/entfernen). „Server erstellen“: Name, Karte, Modus, Instagib, höchstens 2–16 Spieler → startet `elora-server` und verbindet. Hintergrund: Himmel, Wolken, Hügel, die eigene Elora im gewählten Skin und eine zweite. Menümusik aus `assets/music/menu.wav` in Schleife (Lautstärke `music_volume`), aus im Spiel. Debug-Panel mit F1. Bild `docs/design/elora-menue-umgesetzt.png`.
- **M7.4 Einstellungsseiten:** `menu_settings.rs`. Spieler (Name, Körper/Füße/Augen aus der Palette, große Vorschau der Figur), Steuerung (Maus-Empfindlichkeit 10–400 %; Tastenbelegung folgt in M7.5), Grafik (Vollbild, VSync, Kantenglättung, UI-Skalierung 50–200 % in 5-%-Schritten, Kamera-Wackeln, Treffer-Marker), Ton (Gesamt- und Musiklautstärke, Stumm, Hinweis ohne Audiogerät), Sprache (Deutsch/English, Hinweis zu Server-Meldungen). Änderungen wirken sofort (Vollbild, VSync und MSAA zur Laufzeit, Sprache neu geladen) und werden gespeichert – bei Reglern erst beim Loslassen. Bild `docs/design/elora-einstellungen-umgesetzt.png`.
- **M7.5 Tastenbelegung:** `bindings.rs` – 17 Aktionen (Bewegung, Hook, Feuern, drei Waffen, nächste/vorige Waffe, Chat, Team-Chat, Scoreboard, Emote-Rad, Selbstmord, Abstimmung Ja/Nein) auf Taste, Maustaste oder Mausrad, eine Belegung je Aktion (E-117); gespeichert als lesbare Namen unter `[bindings]` (ungültige Einträge → Standard). Fest: Esc (Pause), F1 (Debug-Panel); R und F5 in der Sandbox nur, solange sie keiner Aktion zugeordnet sind. Seite „Steuerung“: Liste in zwei Spalten, Klick → nächste Taste/Maustaste/Radrichtung übernehmen (Esc bricht ab), Doppelbelegung rot, „Standard wiederherstellen“. Knöpfe wählen die Schriftfarbe nach Helligkeit (dunkel auf Sand). Bild `docs/design/elora-steuerung-umgesetzt.png`.
- **M7.6 Server-Info-Abfrage:** `elora-net/src/info.rs` (`InfoProbe`) und neue Paketarten im Endpunkt: Token anfordern (vorhandene, auf 512 Byte aufgefüllte Anfrage – keine Verstärkung), dann `[8][Token][Nonce]` → `[9][Nonce][Info]` nur bei gültigem, adressgebundenem Token, höchstens 20 Antworten je IP und Sekunde; Ping = Zeit zwischen Info-Anfrage und Antwort; Zeitlimit 2 s, Token-Anfrage wird alle 0,5 s wiederholt. LAN-Suche: Token-Anfrage an Broadcast-Adressen (`UdpSocket::set_broadcast`), jeder antwortende Server wird abgefragt. Inhalt (`elora-protocol/src/info.rs`, `ServerInfo`): Protokollversion, Name, Karte, Modus (z. B. `iCTF`), verbundene Menschen, Höchstzahl, bis zu 32 Spieler mit Punkten, Team, Dummy-Kennzeichen (passt in ein Datagramm). Der Server erneuert die Info jede Sekunde. Tests: Ping, Zeitlimit, Suche, falsches Token, Integrationstest mit Spieler und Dummies.
- **M7.8 Master-Server:** `apps/elora-master` (Bibliothek + Programm). Dienst mit `tiny_http`: `POST /register` (`{"port", "version"}`; IP aus der Verbindung bzw. mit `--behind-proxy` aus `X-Forwarded-For`), `GET /servers` (`{"servers": ["IP:Port", …]}`), `GET /`. Aufnahme erst nach erfolgreicher UDP-Info-Abfrage mit passender Protokollversion (`InfoProbe`); Ablauf nach 60 s, Neuanmeldung frühestens nach 5 s, höchstens 32 Server je IP und 8192 insgesamt, Anfragen bis 4 KiB. HTTPS-Client (`ureq` mit rustls, E-127) für Anmelden und Liste holen; Spielserver melden sich mit `--master`/`masters` alle 20 s an (eigener Thread, loggt nur Wechsel). Probelauf lokal: Anmeldung → Prüfung → Liste. Standard-Adresse des öffentlichen Masters offen (O-47).
- **M7.7 Server-Browser:** `browser.rs` (Zustand, Abfragen über eigenen UDP-Socket mit Broadcast, Liste vom Master im eigenen Thread) und `menu_browser.rs` (Oberfläche auf „Spielen“). Reiter Internet (Master-Liste; ohne eingestellten Master Feld zum Eintragen, gespeichert als `master_url`), LAN (Broadcast an 255.255.255.255 und 127.0.0.1, Ports 8303–8310) und Favoriten; Wechsel oder „Aktualisieren“ lädt neu. Sortieren nach Ping, Spielern, Name; Filter „leere/volle ausblenden“. Liste: Name, Karte, Modus, Spieler, Ping; „…“ während der Abfrage, „nicht erreichbar“ nach 2 s, „andere Version“ ohne Verbinden. Details: Spielerliste in Teamfarben mit Punkten und Dummy-Kennzeichen, Verbinden, Favorit merken/entfernen; Doppelklick verbindet; darunter Direkt-Verbinden. Bild `docs/design/elora-browser-umgesetzt.png`.
- **M7.9 Ingame-Menü:** `menu_pause.rs`. Esc (oder Fokusverlust) öffnet die Pause: links Fortsetzen, Einstellungen (alle Einstellungsseiten direkt im Spiel, „Zurück“), Zum Hauptmenü, Beenden; rechts online Serverzeile (Adresse · Modus), Team (Rot/Blau/Zuschauen bzw. Mitspielen/Zuschauen; aktuelles Team farbig), Selbstmord, Abstimmung starten (Karte per Name, Modus mit Instagib, Kick, Zuschauer – Spielerauswahl aus der Liste) bzw. Ja/Nein bei laufender Abstimmung; im Training Respawn. Nach Selbstmord, Abstimmung und Respawn geht es direkt weiter. Das Debug-Panel (F1) bleibt als Entwicklerwerkzeug. Bild `docs/design/elora-pause-umgesetzt.png`.
