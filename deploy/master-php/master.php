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
require __DIR__ . '/common.php';

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

/** Protokollversion eines Servers per UDP-Info-Abfrage, oder `null`. */
function probe(string $ip, int $port, float $timeout): ?int
{
    $info = elora_query(['s' => [$ip, $port]], $timeout)['s'] ?? null;
    return $info === null ? null : (new EloraReader($info))->uvar();
}

$route = $_GET['route'] ?? trim((string) parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH), '/');
$route = basename($route);
$method = $_SERVER['REQUEST_METHOD'] ?? 'GET';

if ($method === 'GET' && $route === 'servers') {
    $list = elora_listed($config);
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
    $key = elora_addr($ip, $port);
    // Grenzen prüfen und Anmeldung vermerken (kurz gesperrt, die UDP-Prüfung läuft danach)
    $refused = elora_store($config, true, function (array &$data, int $now) use ($config, $key, $ip) {
        if (isset($data['last'][$key]) && $now - $data['last'][$key] < $config['min_reregister']) {
            return 'zu häufige Anmeldung';
        }
        if (!isset($data['listed'][$key])) {
            $prefix = elora_addr($ip, 0);
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
    $stored = false;
    elora_store($config, true, function (array &$data, int $now) use ($config, $key) {
        $data['listed'][$key] = $now + $config['expiry'];
    }, $stored);
    if (!$stored) {
        reply(500, ['ok' => false, 'message' => 'Datenablage nicht beschreibbar']);
    }
    reply(200, ['ok' => true, 'message' => 'gelistet']);
}

if ($method === 'GET' && ($route === '' || $route === 'master.php')) {
    reply(200, ['service' => 'elora-master']);
}

reply(404, ['ok' => false, 'message' => 'unbekannt']);
