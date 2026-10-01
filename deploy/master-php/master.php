<?php
/**
 * Elora-Master als PHP-Skript (M8.4, E-162) – gleiche Schnittstelle wie `elora-master`:
 *
 *   POST /register  {"port": 8303, "version": 6}  → Server meldet sich an (alle 20 s)
 *   GET  /servers                                  → {"servers": ["1.2.3.4:8303", …]}
 *
 * Ein Server kommt erst in die Liste, wenn er auf die verbindungslose Info-Abfrage per UDP
 * antwortet und seine Protokollversion passt. Ohne neue Anmeldung fällt er nach `expiry`
 * Sekunden heraus. Die Liste liegt als JSON-Datei vor (kein Datenbankserver nötig).
 */
declare(strict_types=1);

$config = require __DIR__ . '/config.php';

header('Content-Type: application/json; charset=utf-8');
header('Cache-Control: no-store');
header('X-Content-Type-Options: nosniff');

function reply(int $status, array $body): void
{
    http_response_code($status);
    echo json_encode($body, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    exit;
}

/** Adresse des Anfragenden (der Spielserver selbst). */
function client_ip(array $config): ?string
{
    $ip = $_SERVER['REMOTE_ADDR'] ?? '';
    if ($config['trust_forwarded_for'] && !empty($_SERVER['HTTP_X_FORWARDED_FOR'])) {
        $ip = trim(explode(',', (string) $_SERVER['HTTP_X_FORWARDED_FOR'])[0]);
    }
    return filter_var($ip, FILTER_VALIDATE_IP) !== false ? $ip : null;
}

/** `ip:port` wie in Rust (`SocketAddr`): IPv6 in eckigen Klammern. */
function addr(string $ip, int $port): string
{
    return (str_contains($ip, ':') ? "[$ip]" : $ip) . ':' . $port;
}

/** Liste unter Dateisperre lesen und ändern; `$change` bekommt die Daten als Referenz. */
function with_store(array $config, bool $write, callable $change)
{
    $dir = dirname($config['data_file']);
    if (!is_dir($dir)) {
        @mkdir($dir, 0700, true);
    }
    $fh = @fopen($config['data_file'], 'c+');
    if ($fh === false) {
        reply(500, ['ok' => false, 'message' => 'Datenablage nicht beschreibbar']);
    }
    flock($fh, $write ? LOCK_EX : LOCK_SH);
    $raw = stream_get_contents($fh);
    $data = json_decode($raw ?: '', true);
    if (!is_array($data)) {
        $data = ['listed' => [], 'last' => []];
    }
    // Abgelaufenes entfernen
    $now = time();
    $data['listed'] = array_filter($data['listed'] ?? [], fn ($until) => $until > $now);
    $keep = max($config['min_reregister'], $config['expiry']);
    $data['last'] = array_filter($data['last'] ?? [], fn ($t) => $now - $t < $keep);
    $result = $change($data, $now);
    if ($write) {
        ftruncate($fh, 0);
        rewind($fh);
        fwrite($fh, json_encode($data));
        fflush($fh);
    }
    flock($fh, LOCK_UN);
    fclose($fh);
    return $result;
}

/** Vorzeichenlose Zahl im Varint-Format des Spiels (7 Bit je Byte). */
function read_uvar(string $data, int &$pos): ?int
{
    $value = 0;
    for ($shift = 0; $shift < 35; $shift += 7) {
        if ($pos >= strlen($data)) {
            return null;
        }
        $byte = ord($data[$pos++]);
        $value |= ($byte & 0x7f) << $shift;
        if (($byte & 0x80) === 0) {
            return $value;
        }
    }
    return null;
}

/**
 * Info-Abfrage wie `elora_net::InfoProbe`: Token anfordern (512 Byte, gegen Verstärkung),
 * dann Info mit Token und Zufallswert. Liefert die Protokollversion oder null.
 */
function probe(string $ip, int $port, float $timeout): ?int
{
    $sock = @stream_socket_client('udp://' . addr($ip, $port), $errno, $error, $timeout);
    if ($sock === false) {
        return null;
    }
    stream_set_blocking($sock, false);
    $tokenRequest = chr(1) . "ELORA\0\0\x01" . str_repeat("\0", 512 - 9);
    $deadline = microtime(true) + $timeout;
    $nextSend = 0.0;
    $infoRequest = null;
    $nonce = random_bytes(4);
    $version = null;
    while (($now = microtime(true)) < $deadline) {
        if ($now >= $nextSend) {
            @fwrite($sock, $infoRequest ?? $tokenRequest);
            $nextSend = $now + 0.5;
        }
        $wait = max(0.0, min($deadline, $nextSend) - $now);
        $read = [$sock];
        $none = null;
        if (@stream_select($read, $none, $none, (int) $wait, (int) (fmod($wait, 1.0) * 1e6)) < 1) {
            continue;
        }
        $p = @fread($sock, 2048);
        if ($p === false || $p === '') {
            continue;
        }
        if ($infoRequest === null && strlen($p) === 9 && $p[0] === chr(2)) {
            $infoRequest = chr(8) . substr($p, 1, 8) . $nonce;
            @fwrite($sock, $infoRequest);
            $nextSend = microtime(true) + 0.5;
        } elseif ($infoRequest !== null && strlen($p) >= 5 && $p[0] === chr(9)
            && substr($p, 1, 4) === $nonce) {
            $pos = 5;
            $version = read_uvar($p, $pos);
            break;
        }
    }
    fclose($sock);
    return $version;
}

$route = $_GET['route'] ?? trim((string) parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH), '/');
$route = basename($route);
$method = $_SERVER['REQUEST_METHOD'] ?? 'GET';

if ($method === 'GET' && $route === 'servers') {
    $list = with_store($config, false, fn (array &$data) => array_keys($data['listed']));
    sort($list);
    reply(200, ['servers' => array_values($list)]);
}

if ($method === 'POST' && $route === 'register') {
    $body = json_decode((string) file_get_contents('php://input', false, null, 0, 4096), true);
    $ip = client_ip($config);
    if (!is_array($body) || !is_int($body['port'] ?? null) || !is_int($body['version'] ?? null) || $ip === null) {
        reply(400, ['ok' => false, 'message' => 'ungültige Anfrage']);
    }
    if ($body['version'] !== $config['protocol_version']) {
        reply(400, ['ok' => false, 'message' => 'falsche Protokollversion']);
    }
    $port = $body['port'];
    if ($port < 1 || $port > 65535) {
        reply(400, ['ok' => false, 'message' => 'ungültiger Port']);
    }
    $key = addr($ip, $port);
    // Grenzen prüfen und Anmeldung vermerken (kurz gesperrt, die UDP-Prüfung läuft danach)
    $refused = with_store($config, true, function (array &$data, int $now) use ($config, $key, $ip) {
        if (isset($data['last'][$key]) && $now - $data['last'][$key] < $config['min_reregister']) {
            return 'zu häufige Anmeldung';
        }
        if (!isset($data['listed'][$key])) {
            $prefix = addr($ip, 0);
            $prefix = substr($prefix, 0, -1);
            $same = count(array_filter(array_keys($data['listed']), fn ($a) => str_starts_with($a, $prefix)));
            if ($same >= $config['max_per_ip']) {
                return 'zu viele Server von dieser Adresse';
            }
            if (count($data['listed']) >= $config['max_servers']) {
                return 'Liste voll';
            }
        }
        $data['last'][$key] = $now;
        return null;
    });
    if ($refused !== null) {
        reply(429, ['ok' => false, 'message' => $refused]);
    }
    if ($config['verify_udp']) {
        $version = probe($ip, $port, (float) $config['probe_timeout']);
        if ($version === null) {
            reply(400, ['ok' => false, 'message' => 'Server per UDP nicht erreichbar']);
        }
        if ($version !== $config['protocol_version']) {
            reply(400, ['ok' => false, 'message' => 'Server meldet falsche Protokollversion']);
        }
    }
    with_store($config, true, function (array &$data, int $now) use ($config, $key) {
        $data['listed'][$key] = $now + $config['expiry'];
    });
    reply(200, ['ok' => true, 'message' => 'gelistet']);
}

if ($method === 'GET' && ($route === '' || $route === 'master.php')) {
    reply(200, ['service' => 'elora-master']);
}

reply(404, ['ok' => false, 'message' => 'unbekannt']);
