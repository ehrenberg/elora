# Elora – für Entwicklerinnen und Entwickler

Alles rund um Quellcode, Bauen, Testen, Server-Betrieb und Mitarbeit. Für Spielerinnen und Spieler
steht das Wichtigste in der [README](README.md).

**Stand:** 0.9.1 Beta – Mehrspieler aus Release 1 und als Vorschau das Abenteuer „Die verstummten
Quellen“ (Prolog und Kapitel 1–3). Planung: [Roadmap von Release 2](docs/release-2/roadmap.md),
Änderungen: [Release-Notizen](docs/releases/).

## Inhalt

- [Einrichtung unter Arch Linux](#einrichtung-unter-arch-linux)
- [Starten aus dem Quellcode](#starten-aus-dem-quellcode)
- [Prüfen und Testen](#prüfen-und-testen)
- [Server, Konsole und Master](#server-konsole-und-master)
- [Karten](#karten)
- [Sounds austauschen](#sounds-austauschen)
- [Pakete und Releases](#pakete-und-releases)
- [Dateien und Ordner](#dateien-und-ordner)
- [Projektstruktur](#projektstruktur)
- [Arbeitsweise](#arbeitsweise)
- [Fehlerbehebung](#fehlerbehebung)

## Einrichtung unter Arch Linux

### 1. Pakete

```sh
sudo pacman -S --needed git base-devel rustup cargo-deny cargo-nextest
rustup default stable
```

- `rustup` ersetzt das Paket `rust` (pacman fragt nach – mit Ja bestätigen). Die genaue Rust-Version (siehe `rust-toolchain.toml`) lädt rustup beim ersten `cargo`-Aufruf im Projekt automatisch.
- `cargo-deny` (Lizenz- und Sicherheitsprüfung) und `cargo-nextest` (Test-Runner) braucht `cargo xtask check`.

### 2. Grafik (Vulkan) und Fenster

Der Renderer nutzt wgpu, unter Linux standardmäßig Vulkan. Nötig sind der Vulkan-Loader und der Treiber zur Grafikkarte:

```sh
sudo pacman -S --needed vulkan-icd-loader
sudo pacman -S --needed vulkan-radeon    # AMD
# oder: vulkan-intel                     # Intel
# oder: nvidia-utils                     # NVIDIA (proprietär)
```

Fenster laufen unter Wayland und X11 (auf Desktop-Installationen meist schon vorhanden, sonst: `sudo pacman -S --needed wayland libxkbcommon libx11 libxcursor libxrandr libxi`).

Prüfen: `vulkaninfo --summary` (Paket `vulkan-tools`) sollte die Grafikkarte zeigen.

### 3. Ton

Die Tonausgabe läuft über ALSA (`alsa-lib`, auf Desktop-Installationen vorhanden; mit PipeWire zusätzlich `pipewire-alsa`):

```sh
sudo pacman -S --needed alsa-lib pipewire-alsa
```

Ohne Audiogerät startet das Spiel trotzdem – stumm, mit Hinweis im Panel unter *Ton*.

### 4. Bauen

```sh
git clone https://github.com/ehrenberg/elora.git && cd elora
cargo build            # baut Spiel (elora), Server (elora-server) und Master (elora-master)
```

Alle Befehle werden aus dem Projektordner ausgeführt (Karten liegen in `maps/`).

## Starten aus dem Quellcode

```sh
cargo run --bin elora                               # startet ins Hauptmenü
```

### Sandbox und Abkürzungen

Mit einer Karte, `--mode`, `--connect` oder `--abenteuer` auf der Kommandozeile geht es ohne Menü direkt ins Spiel, mit Debug-Panel (F1):

```sh
cargo run --bin elora -- maps/sandbox.emap          # Standardkarte
cargo run --bin elora -- maps/eigene.emap           # andere Karte
cargo run --bin elora -- maps/ctf-test.emap --mode ctf   # Spielmodus gegen Dummies
cargo run --bin elora -- maps/look-test.emap         # Kartenlook: Materialien, Hintergründe, Deko, Animationen
cargo run --bin elora -- maps/tiles-test.emap        # Plattform, Eis, Sprungfeld, Beschleuniger (S = Runter)
cargo run --bin elora -- maps/faehigkeiten-test.emap # Abenteuer-Fähigkeiten (F1 → „Fähigkeiten“ einschalten)
cargo run --bin elora -- --abenteuer 1              # Abenteuer auf Platz 1 (fortsetzen oder neu)
cargo run --bin elora -- --connect 127.0.0.1:8303   # direkt zu einem Server
```

Spielmodi in der Sandbox: `--mode dm|tdm|ctf|lms|lts` (optional `--instagib`) oder im Panel unter *Spiel → Modus*. CTF braucht eine Karte mit Flaggen, z. B. `maps/ctf-test.emap`.

Die Sandbox dient zum Tunen und Testen: Dummies, Pickups, Gegner des Abenteuers, Live-Regler für alle Werte ([Tuning](docs/handbuch/tuning.md)), Hot-Reload der Karte.

### Entwickler-Tasten

| Taste | Aktion |
|---|---|
| F1 | Debug-Panel ein/aus (auch im Menü); Maus wird frei |
| R | Respawn (nur Sandbox ohne Modus) |
| F5 | Aufzeichnung starten/beenden (nur Sandbox, siehe Determinismus-Tests); im Editor: Testspielen |

## Prüfen und Testen

```sh
cargo xtask check          # alles: Formatierung, clippy, Tests, Lizenzen & Advisories
cargo xtask fmt            # Code formatieren
cargo nextest run --workspace --all-features   # nur Tests
```

Nach jeder Änderung muss `cargo xtask check` grün sein, dann wird committet. Jeder Push wird auf GitHub mit demselben Befehl geprüft.

**Determinismus-Tests (Golden-Dateien):** Die Simulation muss bit-genau reproduzierbar bleiben. In der Sandbox zeichnet `F5` die eigenen Eingaben auf; die Datei landet in `crates/elora-sim/tests/recordings/` und wird zum Regressionstest. Golden-Dateien (neu) erzeugen – nur nach bewusster Physik-Änderung:

```sh
ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features
```

**Werkzeuge:**

```sh
cargo xtask net-stats         # Nachrichtengrößen für 8/16/64 Spieler messen
cargo xtask train-huffman    # Huffman-Tabelle neu trainieren (nach Änderungen am Snapshot-Format)
cargo xtask sound-preview    # alle Sounds als WAV nach target/sounds/ (Hörprobe; Parameter in assets/sounds/sounds.toml)
cargo xtask svg-preview a.svg a.png 1200   # SVG rastern (Entwürfe, Assets)
cargo xtask map-dump maps/dm-wiese.emap    # Karte als Text ansehen
```

## Server, Konsole und Master

### Server starten

```sh
cargo run --bin elora-server
cargo run --bin elora-server -- --port 8303 --map maps/dm-wiese.emap --max-clients 16 --name "Mein Server"
cargo run --bin elora-server -- --config server.toml
```

| Option | Standard | Bedeutung |
|---|---|---|
| `--port` | 8303 | UDP-Port |
| `--bind` | `::` | `::` = IPv4 und IPv6 (meldet sich beim Master über beide an), `0.0.0.0` = nur IPv4 |
| `--map` | `maps/dm-wiese.emap` | Karte (relativ: erst Arbeitsverzeichnis, dann Datenordner der Installation) |
| `--max-clients` | 8 | Spieler, 1–64 |
| `--name` | Elora-Server | Anzeigename |
| `--high-bandwidth` | aus | Snapshots mit 50 statt 25 Hz (nur LAN) |
| `--key-file` | `server_key.toml` | dauerhafter Server-Schlüssel (wird beim ersten Start erzeugt) |
| `--tuning` | – | Tuning-Datei (Abschnitt `[physics]` wie `tuning.toml`) |
| `--mode` | dm | Spielmodus: `dm`, `tdm`, `ctf`, `lms`, `lts` |
| `--instagib` | aus | nur Laser, ein Treffer tötet (für alle Modi) |
| `--score-limit` | 20 (CTF: 5 Eroberungen) | Siegpunkte, 0 = aus |
| `--time-limit` | 0 | Zeitlimit in Minuten, 0 = aus |
| `--no-friendly-fire` | an | kein Schaden an Teammitgliedern (Rückstoß bleibt) |
| `--no-votes` | an | Abstimmungen abschalten |
| `--master` | `https://elora.bastianswelt.de` | beim Master-Server anmelden (Internet-Liste); ein eigener ersetzt den Standard, mehrfach möglich |
| `--no-master` | – | nicht in die Internet-Liste eintragen |
| `--config` | – | alle Optionen aus einer TOML-Datei; weitere Optionen überschreiben sie |

Weitere Einstellungen nur in der Konfigurationsdatei: `rotation = ["sandbox", "ctf-test"]` (Kartenrotation), `maps_dir`, und ein Abschnitt `[rules]` (`warmup_secs`, `countdown_secs`, `tdm_respawn_secs`, `team_balance_secs`, `match_swap`, `matches_per_map`).

**Aus dem Client hosten:** Hauptmenü → *Server erstellen*. Der Client schreibt `server.toml` und startet `elora-server` als eigenen Prozess (aus dem Quellcode vorher einmal `cargo build`). Mehr Optionen im Debug-Panel → *Netzwerk* → *Server einrichten …*.

### Konsole

Im Terminal des Servers Befehle eintippen – `help` zeigt alle: `status`, `mode ctf`, `instagib on`, `scorelimit 10`, `timelimit 5`, `friendlyfire off`, `restart`, `map <name>`, `maps`, `kick <slot>`, `ban <slot>`, `spectate <slot>`, `say <text>`, `vote cancel`, `quit`.

Beenden: `quit` + Enter oder Strg+C. Log-Ausgabe steuern: `RUST_LOG=debug cargo run --bin elora-server`.

### Firewall und Sicherheit

Für Spieler aus dem LAN/Internet muss der UDP-Port (Standard 8303) freigegeben sein, z. B. `sudo ufw allow 8303/udp` bzw. `sudo firewall-cmd --add-port=8303/udp`.

Die Verbindung ist verschlüsselt (Noise-Protokoll). Der Client merkt sich den Schlüssel jedes Servers in `known_servers.toml` und warnt, wenn er sich ändert (wie SSH). `server_key.toml` ist geheim und gehört nicht ins Git.

### Master-Server (Internet-Liste)

```sh
cargo run --bin elora-master -- --bind 0.0.0.0:8300                # HTTP auf Port 8300
cargo run --bin elora-master -- --bind 127.0.0.1:8300 --behind-proxy   # hinter Reverse-Proxy
```

Spielserver melden sich beim Standard-Master an (abschaltbar mit `--no-master`, eigener mit `--master <URL>` oder `masters = ["…"]` in `server.toml`; aus dem Client gehostete nur mit „Im Internet anzeigen“), alle 20 s; der Master listet sie erst, wenn er sie selbst per UDP erreicht, und entfernt sie nach 60 s ohne Anmeldung. Öffentlich den Master hinter einem Reverse-Proxy mit HTTPS betreiben (z. B. Caddy: `reverse_proxy 127.0.0.1:8300`) und `--behind-proxy` setzen. Schnittstelle: `GET /servers`, `POST /register` (JSON). Betrieb mit systemd oder Docker: [`docs/handbuch/master-betrieb.md`](docs/handbuch/master-betrieb.md).

Für Webspace ohne eigenen Dienst gibt es Master und Projektseite als PHP: [`deploy/master-php/`](deploy/master-php/LIESMICH.md) (läuft unter `https://elora.bastianswelt.de`). Ändert sich die Protokollversion des Spiels, muss `protocol_version` in `deploy/master-php/config.php` mitziehen.

## Karten

- **Format:** [`docs/handbuch/kartenformat.md`](docs/handbuch/kartenformat.md); ansehen mit `cargo xtask map-dump`.
- **Release-Karten** (M6.10): `dm-wiese` (64×36, 4–8 Spieler), `dm-wueste` (96×48, 8–12), `dm-winter` (128×64, 12–16), `ctf-wald` (150×48, 8–12), `ctf-nacht` (190×64, 12–16). Layouts werden in `tools/design/release_maps/` gebaut und geprüft (`python3 tools/design/release_maps/export.py`), die Dateien schreibt `cargo test -p elora-client --bin elora write_release_maps -- --ignored`.
- **Abenteuer-Karten** (`maps/abenteuer/`) entstehen aus Generatoren in `apps/elora-client/src/editor/` (`prolog.rs`, `kapitel1.rs` bis `kapitel3.rs`) und werden mit `write_prolog_maps`, `write_kapitel1_maps` usw. geschrieben (`cargo test -p elora-client --bin elora <name> -- --ignored`). Ein Test prüft, dass die mitgelieferten Dateien aktuell sind.
- **Inhalte des Abenteuers** (Figuren, Gespräche, Aufgaben, Gegner, Gegenstände) liegen als TOML in `assets/adventure/`: [`docs/handbuch/abenteuer-inhalte.md`](docs/handbuch/abenteuer-inhalte.md).
- **Grafiken** entstehen mit Python-Skripten in `tools/design/` (keine KI-Dienste, E-295).

## Sounds austauschen

Jeder Sound hat einen festen Namen (Liste: `cargo xtask sound-preview` oder `crates/elora-audio/src/cues.rs`, z. B. `jump`, `hammer_fire`, `grenade_explode`). Eine Datei `assets/sounds/files/<name>.wav` ersetzt den Klang gleichen Namens.

1. Datei importieren – jedes Format, das ffmpeg liest; optional Start und Länge in Sekunden:
   ```sh
   cargo xtask sound-import jump ~/Downloads/boing.ogg          # ganze Datei
   cargo xtask sound-import jump ~/Downloads/boing.ogg 0.1 0.3  # ab 0,1 s, 0,3 s lang
   ```
   Der Import wandelt nach Mono/44,1 kHz/16 Bit, entfernt Stille am Anfang, blendet das Ende aus und bringt die Spitze auf −1 dBFS. Längstens 2,5 s (Test).
2. Lautstärke im Spiel: Abschnitt `[gain]` in `assets/sounds/sounds.toml` (1 = unverändert).
3. Quelle und Lizenz in `assets/SOURCES.md` eintragen – nur CC0 oder mit GPL-3.0 verträgliche Lizenzen.
4. Anhören: `cargo xtask sound-preview jump` (nach `target/sounds/`) oder im Spiel (`cargo run --bin elora`, eingebettet beim Bauen).
5. `cargo xtask check`, dann committen.

Zurück zum prozeduralen Klang: die Datei löschen und in `sounds.toml` einen Eintrag mit Schichten anlegen (sonst schlägt der Test fehl).

**Musik** liegt als Ogg Vorbis in `assets/music/` (je Gebiet über `music`, `boss_music`, `party_music` in `assets/adventure/worldmap.toml`); Quellen in `assets/SOURCES.md`, CC-BY-Stücke zusätzlich auf der Seite „Über Elora“ nennen.

## Pakete und Releases

`cargo xtask package --archive` baut ein Release-Paket für das eigene System unter `dist/` (Programme, Karten, Musik, Lizenzen, Symbol; macOS zusätzlich `Elora.app`).

Windows package on Linux (cross build with mingw-w64):

```bash
sudo pacman -S mingw-w64-gcc            # Arch; Debian/Ubuntu: apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo xtask package --archive --target x86_64-pc-windows-gnu   # → dist/elora-<version>-windows-x86_64.zip
```

Start `elora.exe` from the extracted folder: the game data (`maps/`, `assets/`) lies next to it. If Elora stops, `%APPDATA%\Elora\crash.txt` and `elora.log` tell why.

Ein Release:

1. Version in `Cargo.toml` setzen, Release-Notizen `docs/releases/v<version>.md` schreiben.
2. Committen, Tag `v<version>` setzen und pushen.
3. GitHub Actions baut Linux (tar.gz, AppImage), Windows (ZIP) und macOS (DMG) und legt ein Entwurfs-Release mit den Notizen an.
4. Entwurf auf GitHub prüfen und von Hand veröffentlichen; bei Bedarf die Projektseite (`deploy/master-php/`) hochladen.

## Dateien und Ordner

Mitgelieferte Daten (`maps/`, Musik) sucht das Spiel erst im Arbeitsverzeichnis, dann in `ELORA_DATA`, neben dem Programm, in `../share/elora` oder `../Resources` (macOS).

| Datei | Zweck | im Git |
|---|---|---|
| `settings.toml` | Spieler-Einstellungen: Name, Skin, Sprache, Grafik, Ton, Effekte, Maus, Steuerung, Favoriten – im Einstellungsordner, beim Beenden gespeichert | nein, lokal |
| `tuning.toml` | Entwickler-Tuning der Sandbox: Physik und Sichtbereich (Panel → *Speichern*) | nein, lokal |
| `server.toml` | vom Client geschriebene Server-Konfiguration | nein |
| `server_key.toml` | geheimer Server-Schlüssel – nicht weitergeben | nein |
| `known_servers.toml` | bekannte Server-Schlüssel des Clients | nein |
| `maps/*.emap` | Karten | ja |

Einstellungsordner: Linux `~/.config/elora`, Windows `%APPDATA%\Elora`, macOS `~/Library/Application Support/Elora`. Eigene und heruntergeladene Karten und die Spielstände des Abenteuers liegen unter `~/.local/share/elora` (Windows/macOS im Einstellungsordner).

## Projektstruktur

```
crates/elora-sim        deterministische Simulation (Physik, Hook, Waffen, Fähigkeiten, Gegner)
crates/elora-map        Kartenformat und Aufbau der Welt aus einer Karte
crates/elora-render     wgpu-2D-Renderer (lyon-Tessellierung), Kamera, Nachbearbeitung
crates/elora-protocol   Nachrichten, Delta-Snapshots, Huffman
crates/elora-net        UDP-Transport: Handshake, Verschlüsselung, Zuverlässigkeit, Simulator
crates/elora-game       Spielregeln: Modi, Punkte, Runden, Teams, Flaggen
crates/elora-adventure  Abenteuer: Spielstand, Aufgaben, Gespräche, Inventar, Läden, Sitzung
crates/elora-audio      Sounds: prozeduraler Generator, Zuordnung zu Ereignissen, Wiedergabe, Musik
apps/elora-client       das Spiel (Menü, Sandbox, Online, Abenteuer, Editor)
apps/elora-server       dedizierter Server
apps/elora-master       Master-Server für die Internet-Liste
xtask                   Entwicklungsbefehle (cargo xtask …)
maps/                   Karten (Release, Test, Abenteuer)
assets/                 Grafiken, Sounds, Musik, Schriften, Sprachen, Abenteuer-Inhalte
tools/design/           Python-Skripte für Entwürfe und Spielgrafiken
deploy/                 Master als Dienst und als PHP mit Projektseite
docs/                   Handbuch, Release-2-Planung, Release-Notizen, Archiv von Release 1
```

Details: [Architektur](docs/handbuch/architektur.md) · Grundsätze: [`docs/handbuch/grundsaetze.md`](docs/handbuch/grundsaetze.md) · Übersicht der Doku: [`docs/README.md`](docs/README.md).

## Arbeitsweise

- Entscheidungen werden mit Nummer festgehalten (`E-…`): Release 2 in [`docs/release-2/entscheidungen.md`](docs/release-2/entscheidungen.md), Release 1 im [Archiv](docs/archiv/release-1/).
- Meilensteine laufen als Plan → Freigabe → Entwürfe → Umsetzung → Abnahme (z. B. [`docs/release-2/m2-3-plan.md`](docs/release-2/m2-3-plan.md)).
- Commits nach Conventional Commits (`feat(adventure): …`, `fix(client): …`), jeder mit grünem `cargo xtask check`.
- Fremde Inhalte nur CC0 oder mit GPL-3.0 verträglich, immer mit Eintrag in `assets/SOURCES.md`.

## Fehlerbehebung

| Problem | Lösung |
|---|---|
| „kein passender Grafikadapter“ / Fenster startet nicht | Vulkan-Treiber installieren (siehe oben); notfalls OpenGL erzwingen: `WGPU_BACKEND=gl cargo run --bin elora` |
| `elora-server … nicht gefunden` beim Hosten aus dem Client | einmal `cargo build` (baut alle Programme) |
| `Port … nicht verfügbar` | anderer Server läuft bereits auf dem Port → `--port` ändern |
| `cargo xtask check` meldet fehlende Befehle | `cargo-deny` und `cargo-nextest` installieren (siehe Einrichtung) |
| Test „Karte veraltet“ | Generator geändert, Karte nicht neu geschrieben → passenden `write_…_maps`-Test mit `--ignored` ausführen |

## Lizenz

- Code: [GPL-3.0](LICENSE)
- Eigene Grafiken & Sounds: CC-BY-SA 4.0
- Schriften (Inter, JetBrains Mono): SIL Open Font License 1.1
- Fremde Inhalte: [`assets/SOURCES.md`](assets/SOURCES.md) · Bibliotheken: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)
