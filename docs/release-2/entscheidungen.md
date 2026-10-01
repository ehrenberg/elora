# Entscheidungslog – Release 2

Fortsetzung ab **E-200**. Release 1 (E-001 bis E-173) steht im
[Archiv](../archiv/release-1/02-entscheidungen.md), die weiter gültigen Grundsätze in
[`../handbuch/grundsaetze.md`](../handbuch/grundsaetze.md).

## Getroffene Entscheidungen

| # | Datum | Thema | Entscheidung | Begründung / Quelle |
|---|---|---|---|---|
| E-200 | 2026-10-02 | Dokumentation | **Archiv + Handbuch:** Release 1 nach `docs/archiv/release-1/`, gültiges Wissen als Handbuch in `docs/handbuch/`, Release 2 in `docs/release-2/` | Entscheidung Projektinhaber |
| E-201 | 2026-10-02 | Entscheidungslog | Log von Release 1 archiviert, **neuer Log ab E-200**; Grundsätze als Kurzfassung im Handbuch | Entscheidung Projektinhaber |
| E-202 | 2026-10-02 | Schwerpunkte Release 2 | **Spieler & Gemeinschaft, mehr Spielinhalt, Mitspieler-Bots** und ein **Rollenspiel-Abenteuer**: Einzelspieler mit Geschichte, NPCs und Rollenspiel-Elementen (Leveln, Waffen und Fähigkeiten ausbauen …), zusätzlich als Spielmodus | Entscheidung Projektinhaber |
| E-203 | 2026-10-02 | Weltaufbau (O-200) | **Hub mit Gebieten:** ein Dorf als Treffpunkt mit NPCs, Händlern und Aufgaben; Gebiete (z. B. Wald, Wüste, Eisberge, Höhlen) werden nach und nach freigeschaltet | Entscheidung Projektinhaber |
| E-204 | 2026-10-02 | Rollenspiel als Spielmodus (O-201) | **Rollenspiel-PvP-Modus:** eigener Spielmodus, in dem man während des Matches levelt und Waffen/Fähigkeiten ausbaut (Fortschritt gilt für die Runde). Kein Koop-Abenteuer vorgesehen | Entscheidung Projektinhaber |
| E-205 | 2026-10-02 | Geschichte (O-203) | **Claude schlägt vor** (Welt, Figuren, Handlung in Varianten), **Projektinhaber entscheidet** | Entscheidung Projektinhaber |
| E-206 | 2026-10-02 | Rollenspiel-Elemente (O-204) | **Alle vier:** Stufen & Fertigkeiten (Fähigkeitenbaum), Waffen ausbauen, Ausrüstung & Beute (Inventar, Händler, Währung), Aufgaben & Dialoge | Entscheidung Projektinhaber |
| E-207 | 2026-10-02 | Geschichte (O-203) | **Entwurf A „Die verstummten Quellen“** aus [`geschichte-entwuerfe.md`](geschichte-entwuerfe.md): märchenhaft, warm; Dorf Tauwinkel, fünf Gebiete (Blütenwiesen, Murmelwald, Glutsandwüste, Frostspitzen, Sternschlucht), Gegenspieler „Der Dürre“, Versöhnung statt Sieg; PvP-Modus „Quellenkampf“ | Entscheidung Projektinhaber |
| E-208 | 2026-10-02 | Credits: Name | Projektinhaber als **Bastian Ehrenberg** | Entscheidung Projektinhaber |
| E-209 | 2026-10-02 | Credits: Ort (M8.5) | **Letzte Seite in den Einstellungen** („Über Elora“) | Entscheidung Projektinhaber |
| E-210 | 2026-10-02 | Wendung der Geschichte | **Sechste Quelle unter dem Dorfbrunnen**, der Dürre als ihr vergessener Hüter – passt | Entscheidung Projektinhaber |
| E-211 | 2026-10-02 | Altersgruppe | Das Abenteuer ist **immer für 12+ spielbar**: Kämpfe ja, aber ohne Blut und Grausamkeit; die Hüter-Kämpfe dürfen echte Kämpfe sein, die Geschichte erzählt sie als Beruhigen | Entscheidung Projektinhaber |
| E-212 | 2026-10-02 | Namen, Fähigkeiten | Namen von Welt und Figuren sowie die fünf Fähigkeiten in ihrer Reihenfolge (Hook-Ruck, Heranhooken, Stampfen, Eisgriff, Gleiten) **bleiben** | Entscheidung Projektinhaber |
| E-213 | 2026-10-02 | Dialoge | **Auswahl mit Folgen** erlaubt | Entscheidung Projektinhaber |
| E-214 | 2026-10-02 | Spieldauer | Der Einzelspieler-Modus soll **lange tragen – mehrere Stunden** (Zielwerte siehe [Weltbuch §8](weltbuch.md)) | Entscheidung Projektinhaber |

## Offene Punkte

Übernommen aus Release 1:

- [ ] **O-45 Remote-Konsole** (Admin-Befehle aus dem Client mit Passwort)
- [ ] **O-49 Weitere Vertriebskanäle** (itch.io, Flathub, Steam, eigene Website)
- [ ] **O-50 macOS auf Intel** (Cross-Build oder eigener Runner)
- [ ] **O-51 Restpunkte M8:** ~~Credits-Seite~~ (erledigt, E-209); Playtests und Balancing (M8.6) und Tests der Pakete auf Windows/macOS macht der Projektinhaber später
- [ ] **O-52 Demos und Replays** (E-115: nach Release 1)

Neu für Release 2 (Rollenspiel-Abenteuer, siehe [`roadmap.md`](roadmap.md)):

- [x] ~~O-200 Weltaufbau~~ → E-203
- [x] ~~O-201 Rollenspiel als Spielmodus~~ → E-204
- [ ] **O-202 Fortschritt speichern** – Spielstände des Einzelspieler-Abenteuers (z. B. lokal, mehrere Plätze); im PvP-Modus nur für die Runde (E-204)
- [x] ~~O-203 Geschichte und Welt~~ → E-207 · Ausarbeitung: [`weltbuch.md`](weltbuch.md) (Entwurf)
- [ ] **O-204 Fortschrittssystem** – Ausgestaltung von Stufen, Fähigkeitenbaum, Waffen-Ausbau, Beute, Währung (Umfang: E-206)
- [ ] **O-205 NPCs und Gegner** – Verhalten, Dialoge, Händler, Begleiter; Grundlage sind die Bots
- [ ] **O-206 Abenteuer im Editor** – NPCs, Auslöser, Dialoge und Aufgaben in Karten
- [ ] **O-207 Weitere Waffen** – welche, für welche Modi

Für die Zukunft festgehalten:

- [ ] **O-208 Dauerhafte Welt** – ein Server mit fortlaufender Welt, auf dem Spielerfiguren ihren Fortschritt behalten (kleines Online-Rollenspiel; braucht Konten und Speicherung auf dem Server) – nach Release 2
