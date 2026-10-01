# Elora-Master für Webspace (PHP)

Gleiche Schnittstelle wie das Programm `elora-master`, aber als PHP-Skript für normales
Webhosting ohne eigenen Dienst. Client und Server brauchen keine Änderung.

## Voraussetzungen

- PHP **8.0** oder neuer (keine Erweiterungen nötig, kein Datenbankserver)
- HTTPS für die Domain (bei den meisten Hostern per Let's Encrypt im Kundenmenü)
- Apache mit `mod_rewrite` (Standard bei fast allen Webspaces) – sonst siehe unten
- **ausgehendes UDP** muss erlaubt sein (für die Prüfung der Spiel-Server)

## Hochladen

1. Subdomain `elora.bastianswelt.de` im Kundenmenü anlegen, Zielordner z. B. `elora/`.
2. Den **Inhalt** dieses Ordners dorthin hochladen (FTP/SFTP), also:
   `index.php`, `config.php`, `.htaccess`, Ordner `data/` mit seiner `.htaccess`.
   Dateien mit Punkt am Anfang sind in manchen FTP-Programmen versteckt – mit hochladen!
3. Ordner `data/` für PHP beschreibbar machen (meist schon so; sonst Rechte 755 oder 775).
4. HTTPS für die Subdomain einschalten.

## Prüfen

Im Browser oder mit curl:

- `https://elora.bastianswelt.de/` → `{"service":"elora-master"}`
- `https://elora.bastianswelt.de/servers` → `{"servers":[]}`

Dann einen Spiel-Server starten (er meldet sich von selbst an) und nach ein paar Sekunden
erneut `/servers` aufrufen – seine Adresse steht in der Liste. Im Log des Servers steht
`beim Master angemeldet`, sonst die Fehlermeldung des Masters.

## Wenn etwas nicht geht

| Meldung des Servers | Ursache |
|---|---|
| `Server per UDP nicht erreichbar` | Spiel-Server-Port (UDP) nicht von außen erreichbar – oder der Hoster sperrt ausgehendes UDP. Im zweiten Fall in `config.php` `'verify_udp' => false` setzen: Server werden dann ungeprüft gelistet (nur mit ihrer eigenen Adresse, höchstens 32 je Adresse). |
| `falsche Protokollversion` | Spiel und `config.php` (`protocol_version`) passen nicht zusammen – nach einem Update der Spielversion anpassen. |
| `Datenablage nicht beschreibbar` | Ordner `data/` beschreibbar machen. |
| 404 bei `/servers` | `mod_rewrite`/`.htaccess` greift nicht. Ohne Rewrite geht auch die Adresse `https://elora.bastianswelt.de/index.php` als Master-Adresse (in den Einstellungen bzw. `--master`), dann wird `/index.php/servers` aufgerufen. |

## Hinweise

- Steht ein CDN/Proxy (z. B. Cloudflare) vor der Seite, `trust_forwarded_for` auf `true`
  setzen – sonst sieht der Master nur die Adresse des Proxys.
- Die Liste liegt in `data/servers.json` und räumt sich selbst auf (60 s ohne Anmeldung).
