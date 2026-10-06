<p align="center">
  <img src="assets/elora/elora.svg" width="120" alt="Elora">
</p>

<h1 align="center">Elora</h1>

<p align="center">
  <b>Schnelles 2D-Multiplayer: hooken, schwingen, sprengen – mit Freunden im Internet oder im LAN.<br>
  Und jetzt auch allein im Abenteuer.</b>
</p>

<p align="center">
  <a href="https://github.com/ehrenberg/elora/releases"><b>Herunterladen</b></a> ·
  <a href="https://elora.bastianswelt.de">Projektseite und Live-Server</a> ·
  <a href="docs/releases/v0.9.1.md">Was ist neu in 0.9.1?</a> ·
  <a href="https://github.com/ehrenberg/elora/issues">Fehler melden</a>
</p>

---

Elora ist ein schnelles 2D-Spiel im Stil der großen Klassiker des Genres: flinke Bewegung, ein
Haken zum Schwingen und eine Handvoll Waffen, die man in Sekunden versteht und in Wochen meistert.
Elora ist auch der Name der Heldin – ein kleiner, mutiger Tropfen.

**Version 0.9.1 Beta** · kostenlos und Open Source · Linux, Windows, macOS · Deutsch und Englisch

## Was dich erwartet

- **Hook, Hammer, Granatwerfer, Laser** – häng dich an Wände, schwing durch die Karte, zieh Gegner zu dir
- **Fünf Spielmodi:** Deathmatch, Team-Deathmatch, Capture the Flag, Last Man Standing, Last Team Standing – jeweils auch als Instagib
- **Online und im LAN:** Serverliste mit Live-Status, Favoriten, eigener Server mit einem Klick
- **Das Abenteuer „Die verstummten Quellen“** (Vorschau): Prolog und drei Kapitel für dich allein
- **Karten-Editor** direkt im Spiel – bauen, F5 drücken, losspielen
- **Elora nach deinem Geschmack:** Farben für Augen, Körper und Füße, Emotes für schnelle Grüße

## Herunterladen und starten

Die neueste Version gibt es unter **[Releases](https://github.com/ehrenberg/elora/releases)**.

| System | Datei | So geht's |
|---|---|---|
| Linux | `elora-…-linux-x86_64.AppImage` | Rechtsklick → Eigenschaften → „Ausführbar“, dann doppelklicken (oder `chmod +x` im Terminal) |
| Linux | `elora-…-linux-x86_64.tar.gz` | entpacken, `elora` starten |
| Windows | `elora-…-windows-x86_64.zip` | entpacken, `elora.exe` starten |
| macOS (Apple Silicon) | `elora-…-macos-aarch64.dmg` | Elora in „Programme“ ziehen; beim ersten Start **Rechtsklick → Öffnen** (die App ist nicht signiert) |

Elora braucht eine Grafikkarte mit Vulkan, DirectX 12 oder Metal. Für Intel-Macs gibt es noch
kein Paket.

## Erste Schritte

Nach dem Start landest du im Hauptmenü:

| Menü | Was du dort machst |
|---|---|
| **Spielen** | Server im Internet, im LAN oder aus deinen Favoriten finden und beitreten; „Schnell spielen“ bringt dich zum letzten Server |
| **Abenteuer** | das Abenteuer auf einem von drei Spielstand-Plätzen beginnen oder fortsetzen |
| **Training** | allein üben, mit Übungsgegnern und allen Waffen |
| **Server erstellen** | einen eigenen Server starten und gleich mitspielen |
| **Editor** | eigene Karten bauen |
| **Einstellungen** | Name, Aussehen, Steuerung, Grafik, Ton, Sprache |

Im Spiel öffnet **Esc** das Pause-Menü: Team wählen, zuschauen, abstimmen, Einstellungen.

## Das Abenteuer

Die Quellen des Taulands verstummen, und die Farben weichen aus dem Dorf Tauwinkel. Elora zieht
los, um herauszufinden, warum.

- **Prolog – Tauwinkel:** Oma Pfütze, Tüftel, Klonk, Lotte und Pip zeigen dir, was du brauchst
- **Kapitel 1 – Blütenwiesen:** verirrte Bienen und eine sehr schlecht gelaunte Brummbär-Hummel
- **Kapitel 2 – Murmelwald:** ein Uhu voller Geschichten, ein Pilzkind auf dem Heimweg und der Wurzelwächter
- **Kapitel 3 – Glutsandwüste:** Sirups Karawane, Treibsand, flirrende Hitze und die Sandschlange

Jede befreite Quelle bringt eine neue Fähigkeit (Hook-Ruck, Heranhooken, Stampfen) – und damit
neue Wege in Gebieten, die du schon kennst. Dazu gibt es Stufen, einen Fähigkeitenbaum,
Ausrüstung, Läden, Aufgaben und Gespräche. Gespeichert wird beim Kartenwechsel und an den
Quellsteinen. Weitere Kapitel folgen mit den nächsten Versionen.

## Mit anderen spielen

- **Einem Server beitreten:** Hauptmenü → *Spielen*. Die Liste zeigt Server im Internet und im LAN;
  über die Adresse kannst du auch direkt verbinden. Welche Server gerade laufen, siehst du auch auf
  der [Projektseite](https://elora.bastianswelt.de).
- **Selbst einen Server starten:** Hauptmenü → *Server erstellen*, Karte und Modus wählen, starten.
  Mit „Im Internet anzeigen“ erscheint er in der Liste aller Spieler. Damit andere ihn erreichen,
  muss der UDP-Port (Standard **8303**) in Router bzw. Firewall freigegeben sein.
- **Einen Server dauerhaft betreiben:** mit dem Programm `elora-server` – alle Optionen stehen in
  der [Anleitung für Entwickler und Server-Betreiber](DEVELOPMENT.md#server-konsole-und-master).

Die Verbindung ist verschlüsselt. Elora merkt sich jeden Server und warnt, wenn sich sein Schlüssel
ändert.

## Eigene Karten bauen

Hauptmenü → *Editor*: Gelände malen, Materialien wie Erde, Sand, Schnee und Eis wählen, Deko und
Hintergründe setzen, Wolken und Bäume animieren, eigene SVG-Grafiken einbetten. Mit **F5** spielst
du die Karte sofort an, Esc bringt dich zurück. Eigene Karten erscheinen danach in *Training* und
*Server erstellen*; spielt ihr online, verteilt der Server sie automatisch an alle.

## Steuerung

Alles außer Esc lässt sich unter *Einstellungen → Steuerung* umbelegen.

| Taste | Aktion |
|---|---|
| A / D | laufen |
| Leertaste | springen, in der Luft Doppelsprung |
| Rechte Maustaste (halten) | Hook |
| Linke Maustaste | schießen (Granate und Laser: halten für Dauerfeuer) |
| 1 / 2 / 3 oder Mausrad | Hammer / Granate / Laser |
| S | durch Holzstege fallen; in der Luft stampfen (Abenteuer, sobald freigeschaltet) |
| Esc | Pause-Menü |
| Tab (halten) | Punktetafel; im Abenteuer: Abenteuer-Menü (Inventar, Fähigkeiten, Aufgaben, Karte) |
| T / Y | Chat / Team-Chat |
| Strg links (halten) | Emote-Rad: Maus in Richtung des Emotes, loslassen |
| K | Neustart an einem Startpunkt (Selbstmord) |
| F3 / F4 | Ja / Nein bei Abstimmungen |
| E | sprechen, öffnen, benutzen (Abenteuer) |
| Q | Heiltrank trinken (Abenteuer) |
| Shift links | Hook-Ruck (Abenteuer, sobald freigeschaltet) |

Nach dem Tod: Feuertaste für einen schnellen Neustart, sonst geht es nach drei Sekunden von allein
weiter. In Last Man Standing und Last Team Standing wartest du bis zur nächsten Runde.

## Wo liegen meine Daten?

| | Linux | Windows | macOS |
|---|---|---|---|
| Einstellungen | `~/.config/elora` | `%APPDATA%\Elora` | `~/Library/Application Support/Elora` |
| Spielstände und eigene Karten | `~/.local/share/elora` | `%APPDATA%\Elora` | `~/Library/Application Support/Elora` |

Zum Sichern einfach diese Ordner kopieren.

## Wenn etwas nicht klappt

| Problem | Lösung |
|---|---|
| Das Spiel startet nicht oder meldet „kein passender Grafikadapter“ | Grafiktreiber aktualisieren. Unter Linux den Vulkan-Treiber installieren (z. B. `vulkan-radeon`, `vulkan-intel` oder `nvidia-utils`). Notfalls OpenGL erzwingen: `WGPU_BACKEND=gl ./elora` |
| Kein Ton | Elora startet auch ohne Tonausgabe – dann stumm. Unter Linux mit PipeWire hilft meist `pipewire-alsa`. |
| Mein Server taucht nicht in der Internet-Liste auf | UDP-Port 8303 im Router und in der Firewall freigeben; „Im Internet anzeigen“ muss an sein. Hinter DS-Lite erreichen dich nur Spieler mit IPv6. |
| Warnung „Server-Schlüssel geändert“ | Der Server wurde neu eingerichtet – oder jemand gibt sich als er aus. Nur vertrauen, wenn du den Grund kennst. |
| Die Maus lässt sich nicht fangen | ins Spielfeld klicken; manche Wayland-Desktops begrenzen die Maus nur, statt sie zu sperren |

Noch etwas kaputt? Melde es gern unter **[Issues](https://github.com/ehrenberg/elora/issues)**.

## Mitmachen

Elora ist freie Software, geschrieben in Rust. Wie du es aus dem Quellcode baust, testest und
mitentwickelst, steht in **[DEVELOPMENT.md](DEVELOPMENT.md)**. Karten bauen, Fehler melden und
Ideen einbringen ist ausdrücklich erwünscht.

## Lizenz und Dank

- Programm: [GPL-3.0](LICENSE)
- Eigene Grafiken und Sounds: CC-BY-SA 4.0
- Schriften (Inter, JetBrains Mono): SIL Open Font License 1.1
- Musik und Sounds anderer Künstler: [`assets/SOURCES.md`](assets/SOURCES.md) und im Spiel unter
  *Einstellungen → Über Elora*
- Bibliotheken: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)

Inspiriert von [Teeworlds](https://teeworlds.com) – danke an Magnus Auvinen und alle Mitwirkenden.
