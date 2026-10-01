<?php
/**
 * Einmaliger Test (nach dem Prüfen wieder löschen!):
 *   https://elora.bastianswelt.de/udp-test.php            – darf der Webspace UDP senden?
 *   https://elora.bastianswelt.de/udp-test.php?port=8303  – erreicht er deinen Spiel-Server?
 * Der Spiel-Server muss dabei laufen; geprüft wird die Adresse, von der du die Seite aufrufst.
 */
declare(strict_types=1);
require __DIR__ . '/common.php';
header('Content-Type: text/plain; charset=utf-8');

$ip = $_SERVER['REMOTE_ADDR'] ?? '?';
echo "Deine Adresse (wie der Master sie sieht): $ip\n\n";

// 1) Ausgehendes UDP: DNS-Anfrage nach example.com an öffentliche Resolver
$query = "\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01";
foreach (['1.1.1.1', '8.8.8.8'] as $dns) {
    $sock = @stream_socket_client("udp://$dns:53", $errno, $error, 2.0);
    $ok = false;
    if ($sock !== false) {
        stream_set_timeout($sock, 2);
        @fwrite($sock, $query);
        $reply = @fread($sock, 512);
        $ok = is_string($reply) && strlen($reply) > 12 && substr($reply, 0, 2) === "\x12\x34";
        fclose($sock);
    }
    echo "UDP-Test DNS $dns: " . ($ok ? "OK – ausgehendes UDP geht" : "KEINE ANTWORT – UDP evtl. gesperrt") . "\n";
}

// 2) Spiel-Server des Aufrufers
$port = (int) ($_GET['port'] ?? 0);
if ($port > 0 && $port < 65536 && filter_var($ip, FILTER_VALIDATE_IP)) {
    $answers = elora_query(['s' => [$ip, $port]], 2.0);
    if (isset($answers['s'])) {
        $info = elora_decode_info($answers['s']);
        echo "\nSpiel-Server $ip:$port: ERREICHBAR – " . ($info ? "„{$info['name']}“, Version {$info['version']}" : 'Antwort unlesbar') . "\n";
    } else {
        echo "\nSpiel-Server $ip:$port: NICHT ERREICHBAR – UDP-Port im Router/der Firewall freigeben (Weiterleitung auf den Rechner mit dem Server)\n";
    }
}
