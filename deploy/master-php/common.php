<?php
/**
 * Gemeinsame Teile von Master (`master.php`) und Projektseite (`index.php`):
 * Liste der Server (JSON-Datei mit Sperre) und die verbindungslose UDP-Info-Abfrage
 * des Spiels (wie `elora_net::InfoProbe`, mehrere Server gleichzeitig).
 */
declare(strict_types=1);

/** `ip:port` wie in Rust (`SocketAddr`): IPv6 in eckigen Klammern. */
function elora_addr(string $ip, int $port): string
{
    return (str_contains($ip, ':') ? "[$ip]" : $ip) . ':' . $port;
}

/**
 * Liste unter Dateisperre lesen (und bei `$write` ändern). `$change` bekommt die Daten
 * als Referenz und die aktuelle Zeit. Abgelaufene Einträge sind schon entfernt.
 * Liefert den Rückgabewert von `$change`, bei Dateifehlern `null` (siehe `$ok`).
 */
function elora_store(array $config, bool $write, callable $change, ?bool &$ok = null)
{
    $ok = false;
    $dir = dirname($config['data_file']);
    if (!is_dir($dir)) {
        @mkdir($dir, 0700, true);
    }
    $fh = @fopen($config['data_file'], $write ? 'c+' : 'r');
    if ($fh === false) {
        if (!$write) {
            // noch keine Datei = noch kein Server
            $ok = true;
            $data = ['listed' => [], 'last' => []];
            return $change($data, time());
        }
        return null;
    }
    flock($fh, $write ? LOCK_EX : LOCK_SH);
    $data = json_decode(stream_get_contents($fh) ?: '', true);
    if (!is_array($data)) {
        $data = ['listed' => [], 'last' => []];
    }
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
    $ok = true;
    return $result;
}

/** Gelistete Server (`ip:port`), sortiert. */
function elora_listed(array $config): array
{
    $list = elora_store($config, false, fn (array &$data) => array_keys($data['listed'])) ?? [];
    sort($list);
    return $list;
}

/** Lesezeiger über die Info-Daten (Kodierung von `elora-protocol`). */
final class EloraReader
{
    public int $pos = 0;

    public function __construct(private string $data)
    {
    }

    public function uvar(): ?int
    {
        $value = 0;
        for ($shift = 0; $shift < 63; $shift += 7) {
            if ($this->pos >= strlen($this->data)) {
                return null;
            }
            $byte = ord($this->data[$this->pos++]);
            $value |= ($byte & 0x7f) << $shift;
            if (($byte & 0x80) === 0) {
                return $value;
            }
        }
        return null;
    }

    /** ZigZag: kleine Beträge, auch negative, in wenigen Bytes. */
    public function ivar(): ?int
    {
        $v = $this->uvar();
        return $v === null ? null : (($v >> 1) ^ -($v & 1));
    }

    public function u8(): ?int
    {
        return $this->pos < strlen($this->data) ? ord($this->data[$this->pos++]) : null;
    }

    public function str(int $max): ?string
    {
        $n = $this->uvar();
        if ($n === null || $n > $max || $this->pos + $n > strlen($this->data)) {
            return null;
        }
        $s = substr($this->data, $this->pos, $n);
        $this->pos += $n;
        // gültiges UTF-8 (ohne mbstring-Erweiterung)
        return preg_match('//u', $s) === 1 ? $s : null;
    }
}

/** Server-Info (`elora_protocol::ServerInfo`) lesen; `null` bei kaputten Daten. */
function elora_decode_info(string $data): ?array
{
    $r = new EloraReader($data);
    $info = [
        'version' => $r->uvar(),
        'name' => $r->str(128),
        'map' => $r->str(128),
        'mode' => $r->str(64),
        'clients' => $r->uvar(),
        'max_clients' => $r->uvar(),
        'players' => [],
    ];
    $n = $r->uvar();
    if (in_array(null, $info, true) || $n === null || $n > 32) {
        return null;
    }
    $teams = ['none', 'red', 'blue', 'spectator'];
    for ($i = 0; $i < $n; $i++) {
        $name = $r->str(64);
        $score = $r->ivar();
        $team = $r->u8();
        $dummy = $r->u8();
        if ($name === null || $score === null || $team === null || $team > 3 || $dummy === null) {
            return null;
        }
        $info['players'][] = ['name' => $name, 'score' => $score, 'team' => $teams[$team], 'dummy' => $dummy === 1];
    }
    return $info;
}

/**
 * Info-Abfrage an mehrere Server gleichzeitig: Token anfordern (512 Byte, gegen
 * Verstärkung), dann Info mit Token und Zufallswert. Liefert `[ip:port => Info-Bytes]`
 * für alle, die rechtzeitig antworten.
 *
 * @param array<string, array{0: string, 1: int}> $targets Schlüssel → [ip, port]
 */
function elora_query(array $targets, float $timeout): array
{
    $tokenRequest = chr(1) . "ELORA\0\0\x01" . str_repeat("\0", 512 - 9);
    $open = [];
    foreach ($targets as $key => [$ip, $port]) {
        $sock = @stream_socket_client('udp://' . elora_addr($ip, $port), $errno, $error, $timeout);
        if ($sock === false) {
            continue;
        }
        stream_set_blocking($sock, false);
        $open[$key] = ['sock' => $sock, 'info' => null, 'nonce' => random_bytes(4), 'next' => 0.0];
    }
    $answers = [];
    $deadline = microtime(true) + $timeout;
    while ($open && ($now = microtime(true)) < $deadline) {
        $next = $deadline;
        foreach ($open as &$q) {
            if ($now >= $q['next']) {
                @fwrite($q['sock'], $q['info'] ?? $tokenRequest);
                $q['next'] = $now + 0.5;
            }
            $next = min($next, $q['next']);
        }
        unset($q);
        $read = array_column($open, 'sock');
        $none = null;
        $wait = max(0.0, $next - microtime(true));
        if (@stream_select($read, $none, $none, (int) $wait, (int) (fmod($wait, 1.0) * 1e6)) < 1) {
            continue;
        }
        foreach ($open as $key => &$q) {
            if (!in_array($q['sock'], $read, true)) {
                continue;
            }
            $p = @fread($q['sock'], 2048);
            if ($p === false || $p === '') {
                continue;
            }
            if ($q['info'] === null && strlen($p) === 9 && $p[0] === chr(2)) {
                $q['info'] = chr(8) . substr($p, 1, 8) . $q['nonce'];
                @fwrite($q['sock'], $q['info']);
                $q['next'] = microtime(true) + 0.5;
            } elseif ($q['info'] !== null && strlen($p) >= 5 && $p[0] === chr(9)
                && substr($p, 1, 4) === $q['nonce']) {
                $answers[$key] = substr($p, 5);
                fclose($q['sock']);
                unset($open[$key]);
            }
        }
        unset($q);
    }
    foreach ($open as $q) {
        fclose($q['sock']);
    }
    return $answers;
}

/** `ip:port` (auch `[v6]:port`) zerlegen. */
function elora_split(string $addr): ?array
{
    if (preg_match('/^\[(.+)\]:(\d+)$/', $addr, $m) || preg_match('/^([^:]+):(\d+)$/', $addr, $m)) {
        return [$m[1], (int) $m[2]];
    }
    return null;
}
