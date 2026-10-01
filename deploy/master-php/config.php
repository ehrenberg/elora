<?php
// Einstellungen des Elora-Masters (PHP-Fassung für Webspace, M8.4).
return [
    // Protokollversion des Spiels; nur Server mit genau dieser Version kommen in die Liste.
    // Bei einer neuen Spielversion mit anderem Protokoll hier anpassen.
    'protocol_version' => 6,

    // Server per UDP prüfen, bevor sie gelistet werden (empfohlen). Manche Webhoster
    // sperren ausgehendes UDP – dann hier false setzen (Liste ungeprüft, siehe LIESMICH.md).
    'verify_udp' => true,

    // Wartezeit der UDP-Prüfung in Sekunden.
    'probe_timeout' => 2.0,

    // Nur auf true setzen, wenn ein Proxy/CDN (z. B. Cloudflare) vor dem Webspace steht
    // und X-Forwarded-For zuverlässig setzt. Sonst könnte jeder fremde Adressen eintragen.
    'trust_forwarded_for' => false,

    // Grenzen wie beim Rust-Master.
    'expiry' => 60,          // Sekunden ohne neue Anmeldung bis zum Austragen
    'min_reregister' => 5,   // Mindestabstand zweier Anmeldungen derselben Adresse
    'max_per_ip' => 32,
    'max_servers' => 8192,

    // Ablage der Liste (Ordner muss für PHP beschreibbar sein, ist per .htaccess gesperrt).
    'data_file' => __DIR__ . '/data/servers.json',
];
