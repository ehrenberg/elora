<?php
/**
 * Projektseite von Elora mit Live-Status der Server (M8.4).
 *
 * Die Server-Liste kommt vom Master in diesem Ordner (`data/servers.json`); Name, Karte,
 * Modus und Spieler fragt die Seite bei den Servern selbst per UDP ab (alle gleichzeitig)
 * und merkt sich das Ergebnis kurz (`data/status.json`), damit Aufrufe die Server nicht
 * belasten. Alle Texte der Server werden vor der Ausgabe maskiert.
 */
declare(strict_types=1);

$config = require __DIR__ . '/config.php';
require __DIR__ . '/common.php';
date_default_timezone_set('Europe/Berlin');

const GITHUB = 'https://github.com/ehrenberg/elora';
const STATUS_TTL = 15;      // Sekunden bis zur nächsten Abfrage der Server
const STATUS_MAX = 48;      // höchstens so viele Server abfragen
const STATUS_TIMEOUT = 1.2; // Sekunden für die UDP-Abfrage

function h(string $s): string
{
    return htmlspecialchars($s, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

/** Server-Status (zwischengespeichert). */
function server_status(array $config): array
{
    $cache = dirname($config['data_file']) . '/status.json';
    $cached = @json_decode((string) @file_get_contents($cache), true);
    if (is_array($cached) && ($cached['time'] ?? 0) > time() - STATUS_TTL) {
        return $cached;
    }
    $targets = [];
    foreach (array_slice(elora_listed($config), 0, STATUS_MAX) as $addr) {
        if ($split = elora_split($addr)) {
            $targets[$addr] = $split;
        }
    }
    $servers = [];
    foreach (elora_query($targets, STATUS_TIMEOUT) as $addr => $data) {
        $info = elora_decode_info($data);
        if ($info === null || $info['version'] !== $config['protocol_version']) {
            continue;
        }
        $humans = array_values(array_filter($info['players'], fn ($p) => !$p['dummy']));
        $servers[] = [
            'addr' => $addr,
            'name' => $info['name'],
            'map' => $info['map'],
            'mode' => $info['mode'],
            'clients' => $info['clients'],
            'max' => $info['max_clients'],
            'players' => array_map(fn ($p) => ['name' => $p['name'], 'team' => $p['team']], array_slice($humans, 0, 8)),
        ];
    }
    // derselbe Server über IPv4 und IPv6 gelistet: nur einmal zeigen
    $seen = [];
    $servers = array_values(array_filter($servers, function ($s) use (&$seen) {
        $port = elora_split($s['addr'])[1] ?? 0;
        $key = $port . '|' . $s['name'] . '|' . $s['map'];
        if (isset($seen[$key])) {
            return false;
        }
        $seen[$key] = true;
        return true;
    }));
    usort($servers, fn ($a, $b) => [$b['clients'], $a['name']] <=> [$a['clients'], $b['name']]);
    $status = ['time' => time(), 'servers' => $servers];
    @file_put_contents($cache, json_encode($status), LOCK_EX);
    return $status;
}

/** SVG aus `assets/` direkt einbetten (für Umfärben per CSS). */
function svg(string $name, string $class = ''): string
{
    $svg = (string) @file_get_contents(__DIR__ . "/assets/$name.svg");
    $svg = preg_replace('/<!--.*?-->/s', '', $svg);
    return '<span class="svg ' . h($class) . '" aria-hidden="true">' . $svg . '</span>';
}

function img(string $name, string $class = '', string $alt = ''): string
{
    return '<img src="assets/' . h($name) . '.svg" class="' . h($class) . '" alt="' . h($alt) . '" loading="lazy">';
}

/** Hautfarben wie im Spiel: Körper, Bauch (aufgehellt), Füße. */
const SKINS = [
    'sun' => ['#f2c14e', '#f8dd9e', '#d9a43a'],
    'blue' => ['#5aaee8', '#b4dcf6', '#4f6fd0'],
    'red' => ['#e0574f', '#f2a8a2', '#a8403a'],
    'green' => ['#7ccf8a', '#c3ebc9', '#4f9c5e'],
    'violet' => ['#b18be8', '#ddcaf6', '#7e5fc4'],
    'pink' => ['#ef7fb0', '#f8c4db', '#c85d8d'],
    'mint' => ['#6fd6c8', '#bfeee7', '#3f9f94'],
];

/** Elora in einer Hautfarbe als Bild (umgefärbtes SVG), optional mit Emote darüber. */
function elora(string $skin, string $class = '', ?string $emote = null): string
{
    static $cache = [];
    if (!isset($cache[$skin])) {
        $svg = (string) @file_get_contents(__DIR__ . '/assets/elora.svg');
        [$body, $belly, $feet] = SKINS[$skin] ?? SKINS['sun'];
        $svg = str_replace(['#f2c14e', '#f8dd9e', '#d9a43a'], [$body, $belly, $feet], $svg);
        $cache[$skin] = 'data:image/svg+xml;base64,' . base64_encode($svg);
    }
    $bubble = $emote === null ? '' :
        '<span class="emote">' . img('bubble') . img($emote) . '</span>';
    return '<span class="elora ' . h($class) . '">' . $bubble
        . '<img class="body" src="' . $cache[$skin] . '" alt=""></span>';
}

$status = server_status($config);
$servers = $status['servers'];
$playersOnline = array_sum(array_column($servers, 'clients'));
$checked = date('H:i', (int) $status['time']);
?>
<!doctype html>
<html lang="de">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Elora – schnelles 2D-Multiplayer mit Hook, Hammer und Granate – und ein Abenteuer</title>
<meta name="description" content="Elora ist ein schnelles 2D-Multiplayer-Spiel: hooken, schwingen, sprengen – jetzt auch mit dem Abenteuer „Die verstummten Quellen“. Open Source, mit Karten-Editor, für Linux, Windows und macOS.">
<link rel="icon" href="assets/elora.svg" type="image/svg+xml">
<style>
@font-face { font-family: "Inter"; src: url("assets/Inter-Regular.ttf") format("truetype"); font-display: swap; }
:root {
  --ink: #1e2a36; --ink-soft: #4b5b6b; --line: #2b2b2b;
  --sky-top: #a9cde8; --sky-bottom: #e6f2f8;
  --earth: #a87a52; --earth-dark: #8a6040; --grass: #7bbf55; --grass-dark: #5f9c42;
  --sun: #f2c14e; --red: #e0574f; --blue: #4f86e0; --green: #6fbf73; --violet: #9b7fe0; --pink: #ef7fb0;
  --card: #ffffff; --page: #f3f7fb; --shadow: 0 10px 30px rgba(30, 42, 54, .10), 0 2px 6px rgba(30, 42, 54, .06);
  --radius: 22px;
  --head: ui-rounded, "SF Pro Rounded", "Nunito", system-ui, -apple-system, "Segoe UI", sans-serif;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; background: var(--page); color: var(--ink); font: 17px/1.6 "Inter", system-ui, sans-serif; overflow-x: hidden; }
a { color: inherit; }
h1, h2, h3 { font-family: var(--head); line-height: 1.15; margin: 0 0 .4em; letter-spacing: -.01em; }
h2 { font-size: clamp(28px, 4vw, 40px); font-weight: 800; }
h3 { font-size: 21px; font-weight: 700; }
p { margin: 0 0 1em; color: var(--ink-soft); }
.svg { display: inline-block; line-height: 0; }
.svg svg { width: 100%; height: 100%; overflow: visible; }
.wrap { width: min(1120px, 100% - 32px); margin: 0 auto; }

/* ---------- Hero: Himmel, Ebenen, Wiese ---------- */
.hero { position: relative; min-height: 860px; background: linear-gradient(var(--sky-top), var(--sky-bottom) 78%); overflow: hidden; isolation: isolate; }
.hero .intro { position: relative; z-index: 5; text-align: center; padding: 64px 16px 0; }
.badge { display: inline-flex; gap: 8px; align-items: center; background: rgba(255,255,255,.75); border-radius: 999px; padding: 6px 14px; font-size: 14px; color: var(--ink-soft); box-shadow: 0 2px 8px rgba(30,42,54,.08); backdrop-filter: blur(6px); }
.badge b { color: var(--ink); white-space: nowrap; }
.title { font-family: var(--head); font-weight: 900; font-size: clamp(64px, 12vw, 132px); margin: 14px 0 0; color: #fff; letter-spacing: -.02em;
  -webkit-text-stroke: 6px var(--line); paint-order: stroke fill; text-shadow: 0 8px 0 rgba(43,43,43,.18); }
.tagline { font-size: clamp(19px, 2.4vw, 24px); color: var(--ink); max-width: 640px; margin: 10px auto 26px; }
.buttons { display: flex; gap: 14px; justify-content: center; flex-wrap: wrap; }
.btn { display: inline-flex; align-items: center; gap: 10px; text-decoration: none; font-family: var(--head); font-weight: 800; font-size: 18px; padding: 14px 24px; border-radius: 16px;
  border: 3px solid var(--line); box-shadow: 0 5px 0 var(--line); transition: transform .12s, box-shadow .12s; }
.btn:hover { transform: translateY(-2px); box-shadow: 0 7px 0 var(--line); }
.btn:active { transform: translateY(3px); box-shadow: 0 2px 0 var(--line); }
.btn.primary { background: var(--sun); color: var(--line); }
.btn.secondary { background: #fff; color: var(--line); }
.btn svg { width: 22px; height: 22px; }

.layer { position: absolute; left: -5%; right: -5%; bottom: 0; background-repeat: repeat-x; background-position: 0 100%; will-change: transform; pointer-events: none; }
.layer.mountains { height: 380px; bottom: 150px; background-image: url("assets/mountains.svg"); background-size: auto 380px; opacity: .95; }
.layer.hills-far { height: 260px; bottom: 120px; background-image: url("assets/hills-far.svg"); background-size: auto 260px; }
.layer.forest { height: 300px; bottom: 105px; background-image: url("assets/forest.svg"); background-size: auto 300px; }
.layer.hills-near { height: 200px; bottom: 95px; background-image: url("assets/hills-near.svg"); background-size: auto 200px; }
.cloud { position: absolute; z-index: 1; opacity: .95; animation: drift linear infinite; will-change: transform; }
@keyframes drift { from { transform: translateX(-30vw); } to { transform: translateX(130vw); } }

.ground { position: absolute; z-index: 3; left: 0; right: 0; bottom: 0; height: 120px; background: var(--earth);
  background-image: radial-gradient(ellipse 18px 12px at 20% 60%, var(--earth-dark) 98%, transparent), radial-gradient(ellipse 10px 7px at 70% 35%, var(--earth-dark) 98%, transparent);
  background-size: 240px 120px, 180px 120px; border-top: 4px solid var(--line); }
.ground::before { content: ""; position: absolute; left: 0; right: 0; top: -4px; height: 26px;
  background: radial-gradient(circle at 8px 10px, var(--grass) 9px, transparent 10px) 0 6px / 16px 20px repeat-x, linear-gradient(var(--grass), var(--grass)) 0 0 / 100% 12px no-repeat;
  border-top: 4px solid var(--line); }
.scene { position: absolute; z-index: 4; left: 0; right: 0; bottom: 116px; height: 0; }
.scene > * { position: absolute; bottom: 0; translate: -50% 0; }
.elora { display: inline-block; position: relative; line-height: 0; }
.elora .body { display: block; width: 100%; height: auto; }
.scene > .elora { position: absolute; }
.scene .elora.big { width: 170px; left: 50%; animation: hop 3.2s ease-in-out infinite; transform-origin: 50% 100%; }
.scene .elora.left { width: 108px; left: 31%; animation: hop 2.6s .4s ease-in-out infinite; }
.scene .elora.right { width: 108px; left: 69%; animation: hop 2.9s 1s ease-in-out infinite; }
.scene .elora.right .body { transform: scaleX(-1); }
@keyframes hop { 0%, 70%, 100% { translate: -50% 0; scale: 1 1; } 78% { scale: 1.06 .92; } 86% { translate: -50% -26px; scale: .96 1.05; } 94% { translate: -50% 0; scale: 1.04 .95; } }
.emote { position: absolute; left: 62%; bottom: 88%; width: 46%; aspect-ratio: 24 / 30; animation: bob 2.4s ease-in-out infinite; }
.emote img { position: absolute; inset: 0; width: 100%; height: 100%; }
@keyframes bob { 50% { transform: translateY(-8px); } }
.pickup { width: 40px; animation: bob 2s ease-in-out infinite; }
.deco { pointer-events: none; }

/* ---------- Abschnitte ---------- */
section { padding: 84px 0; }
.lead { font-size: 19px; max-width: 720px; }
.card { background: var(--card); border-radius: var(--radius); box-shadow: var(--shadow); }

.status { margin-top: -70px; position: relative; z-index: 10; padding: 0; }
.status .card { padding: 28px clamp(18px, 3vw, 36px); }
.status-head { display: flex; flex-wrap: wrap; gap: 18px 32px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
.live { display: inline-flex; align-items: center; gap: 10px; font-family: var(--head); font-weight: 800; font-size: 22px; }
.dot { width: 12px; height: 12px; border-radius: 50%; background: var(--green); box-shadow: 0 0 0 0 rgba(111,191,115,.6); animation: pulse 2s infinite; }
.dot.off { background: #b0b8c1; animation: none; }
@keyframes pulse { 70% { box-shadow: 0 0 0 12px rgba(111,191,115,0); } 100% { box-shadow: 0 0 0 0 rgba(111,191,115,0); } }
.stats { display: flex; gap: 12px; flex-wrap: wrap; }
.stat { background: var(--page); border-radius: 14px; padding: 8px 16px; font-size: 15px; color: var(--ink-soft); }
.stat b { font-family: var(--head); font-size: 22px; color: var(--ink); margin-right: 6px; }
.servers { display: grid; gap: 10px; }
.server { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; gap: 6px 18px; align-items: center; padding: 14px 18px; border-radius: 16px; background: var(--page); }
.server .name { font-weight: 600; color: var(--ink); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.server .meta { font-size: 14px; color: var(--ink-soft); }
.chip { display: inline-block; font-family: var(--head); font-weight: 800; font-size: 13px; letter-spacing: .03em; padding: 4px 10px; border-radius: 999px; background: #e4ecf5; color: var(--ink); }
.fill { width: 110px; height: 10px; border-radius: 99px; background: #dde6ef; overflow: hidden; }
.fill i { display: block; height: 100%; background: linear-gradient(90deg, var(--green), var(--sun)); border-radius: 99px; }
.count { font-variant-numeric: tabular-nums; font-size: 15px; color: var(--ink-soft); min-width: 52px; text-align: right; }
.players { grid-column: 1 / -1; display: flex; flex-wrap: wrap; gap: 6px; }
.p { font-size: 13px; padding: 2px 10px; border-radius: 999px; background: #fff; }
.p.red { box-shadow: inset 3px 0 0 var(--red); } .p.blue { box-shadow: inset 3px 0 0 var(--blue); }
.empty { display: flex; gap: 22px; align-items: center; padding: 10px 4px; }
.empty .elora { width: 90px; flex: none; }
.empty p { margin: 0; }
.note { font-size: 13px; color: #8696a6; margin-top: 14px; }

.features { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 20px; margin-top: 30px; }
.feature { padding: 26px; transition: transform .2s; }
.feature:hover { transform: translateY(-4px); }
.icon { width: 76px; height: 76px; border-radius: 22px; display: grid; place-items: center; margin-bottom: 16px; border: 3px solid var(--line); box-shadow: 0 4px 0 var(--line); }
.icon img, .icon svg { width: 52px; height: 52px; }
.icon.y { background: #fff1c9; } .icon.g { background: #dff3d6; } .icon.b { background: #dce9fb; } .icon.v { background: #ebe3fb; } .icon.p { background: #fbe1ec; }
.feature p { margin: 0; font-size: 16px; }

.modes { display: flex; flex-wrap: wrap; gap: 14px; margin-top: 28px; }
.mode { display: flex; align-items: center; gap: 14px; padding: 14px 20px 14px 14px; }
.mode .tag, .chapter .tag { font-family: var(--head); font-weight: 900; font-size: 18px; width: 64px; height: 48px; display: grid; place-items: center; border-radius: 14px; color: #fff; border: 3px solid var(--line); }
.mode div { line-height: 1.3; } .mode small { color: var(--ink-soft); font-size: 14px; }

.maps { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 18px; margin-top: 30px; }
.map { overflow: hidden; }
.map .pic { position: relative; height: 150px; overflow: hidden; border-bottom: 3px solid var(--line); }
.map .pic .floor { position: absolute; left: 0; right: 0; bottom: 0; height: 34px; border-top: 3px solid var(--line); }
.map .pic img { position: absolute; bottom: 34px; }
.map .txt { padding: 16px 18px 18px; }
.map .txt p { margin: 0; font-size: 15px; }
.map .txt h3 { margin-bottom: 4px; }
.m-wiese .pic { background: linear-gradient(#a9cde8, #e6f2f8); } .m-wiese .floor { background: var(--earth); box-shadow: inset 0 9px 0 var(--grass); }
.m-wueste .pic { background: linear-gradient(#8fbcdf, #f6e2bf); } .m-wueste .floor { background: #d9b26f; box-shadow: inset 0 7px 0 #ecd08f; }
.m-winter .pic { background: linear-gradient(#c9dcec, #f2f6fa); } .m-winter .floor { background: #8c7b70; box-shadow: inset 0 10px 0 #f4f8fb; }
.m-wald .pic { background: linear-gradient(#a9cde8, #e6f2f8); } .m-wald .floor { background: var(--earth); box-shadow: inset 0 9px 0 var(--grass); }
.m-nacht .pic { background: url("assets/stars.svg") 0 0 / 400px auto, linear-gradient(#1f2a55, #4a5a8c); } .m-nacht .floor { background: #566068; box-shadow: inset 0 6px 0 #a5aeb6; }
.m-nacht .pic img { filter: brightness(.62) saturate(.8); }

.editor { display: grid; grid-template-columns: 1.1fr 1fr; gap: 40px; align-items: center; }
.ticks { list-style: none; padding: 0; margin: 0; display: grid; gap: 10px; }
.ticks li { padding-left: 34px; position: relative; color: var(--ink-soft); }
.ticks li::before { content: ""; position: absolute; left: 0; top: 4px; width: 20px; height: 20px; border-radius: 7px; background: var(--grass); border: 2.5px solid var(--line); }
.ticks b { color: var(--ink); }
.new { display: inline-block; background: var(--sun); color: var(--line); font-weight: 700; font-size: 13px; border-radius: 999px; padding: 3px 12px; margin-bottom: 10px; border: 2.5px solid var(--line); }
.chapters { display: grid; gap: 12px; margin: 22px 0 0; }
.chapter { display: flex; gap: 14px; align-items: flex-start; padding: 14px 18px; }
.chapter .tag { flex: none; width: 44px; height: 44px; font-size: 17px; border-radius: 12px; }
.chapter p { margin: 2px 0 0; color: var(--ink-soft); font-size: 15px; }
.adv-scene { position: relative; height: 330px; overflow: hidden; background: linear-gradient(#f7d9a8, #fbeedc); }
.adv-scene .sun { position: absolute; right: 12%; top: 34px; width: 74px; height: 74px; border-radius: 50%; background: #ffd27a; box-shadow: 0 0 0 14px rgba(255,210,122,.35), 0 0 0 30px rgba(255,210,122,.15); }
.adv-scene .floor { position: absolute; left: 0; right: 0; bottom: 0; height: 64px; background: var(--earth); border-top: 3px solid var(--line); box-shadow: inset 0 10px 0 var(--grass); }
.adv-scene .fig { position: absolute; bottom: calc(64px - 17px); width: 200px; }
.adv-scene .fig.flip { transform: scaleX(-1); }
.adv-scene .elora { left: 36%; position: absolute; bottom: calc(64px - 6px); width: 72px; animation: hop 2.8s ease-in-out infinite; transform-origin: 50% 100%; }
.panel { position: relative; height: 330px; overflow: hidden; background: linear-gradient(#a9cde8, #e6f2f8); }
.panel .grid { position: absolute; inset: 0; background-image: linear-gradient(rgba(255,255,255,.35) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.35) 1px, transparent 1px); background-size: 32px 32px; }
.panel .blk { position: absolute; background: var(--earth); border: 3px solid var(--line); border-radius: 10px; box-shadow: inset 0 10px 0 var(--grass); }
.panel .sel { position: absolute; border: 3px dashed #5aaee8; background: rgba(90,174,232,.14); border-radius: 6px; }
.panel .side { position: absolute; right: 0; top: 0; bottom: 0; width: 34%; background: #1b1d22; color: #c9d1d9; font-size: 12px; padding: 14px; line-height: 1.9; }
.panel .side b { color: #fff; display: block; margin: 6px 0 2px; font-size: 13px; }
.panel .side span { display: inline-block; padding: 0 7px; margin: 0 2px 4px 0; border-radius: 5px; background: #2d323a; }
.panel .side span.on { background: #3f6fb5; color: #fff; }

.skins { display: flex; justify-content: center; align-items: flex-end; gap: clamp(10px, 3vw, 40px); flex-wrap: wrap; margin-top: 40px; padding-top: 50px; }
.skins .elora { width: clamp(80px, 12vw, 120px); }
.skins .elora:nth-child(even) .body { transform: scaleX(-1); }

.oss { display: grid; grid-template-columns: auto 1fr auto; gap: 28px; align-items: center; padding: 34px; }
.oss .elora { width: 110px; }
.oss .badges { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px; }

footer { position: relative; margin-top: 40px; padding: 120px 0 40px; color: #c9d4ee; background: url("assets/stars.svg") 0 0 / 700px auto, linear-gradient(#26305e, #141a38); overflow: hidden; }
footer .moon { position: absolute; width: 120px; right: 12%; top: 40px; }
footer p { color: #aab6d6; font-size: 15px; margin: 0 0 6px; }
footer a { color: #fff; }
footer .row { display: flex; justify-content: space-between; gap: 20px; flex-wrap: wrap; align-items: flex-end; }
footer .logo { font-family: var(--head); font-weight: 900; font-size: 34px; color: #fff; }

.reveal { opacity: 0; transform: translateY(24px); transition: opacity .6s ease, transform .6s ease; }
.reveal.shown { opacity: 1; transform: none; }

@media (max-width: 820px) {
  .hero { min-height: 780px; }
  .scene .elora.big { width: 112px; }
  .scene .elora.left { width: 70px; left: 22%; }
  .scene .elora.right { width: 70px; left: 78%; }
  .scene .pickup { display: none; }
  .badge { font-size: 13px; }
  .scene .hide-sm { display: none; }
  .editor { grid-template-columns: 1fr; }
  .adv-scene .fig.hide-sm { display: none; }
  .oss { grid-template-columns: 1fr; text-align: center; }
  .oss .elora { margin: 0 auto; }
  .oss .badges { justify-content: center; }
  .server { grid-template-columns: 1fr auto; }
  .server .fill { display: none; }
}
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation: none !important; transition: none !important; }
  .reveal { opacity: 1; transform: none; }
}
</style>
</head>
<body>

<header class="hero">
  <img src="assets/cloud-1.svg" class="cloud" style="top:70px;width:180px;animation-duration:95s;animation-delay:-20s" alt="">
  <img src="assets/cloud-2.svg" class="cloud" style="top:150px;width:240px;animation-duration:130s;animation-delay:-70s" alt="">
  <img src="assets/cloud-3.svg" class="cloud" style="top:40px;width:150px;animation-duration:110s;animation-delay:-45s" alt="">
  <img src="assets/cloud-1.svg" class="cloud" style="top:210px;width:130px;animation-duration:150s;animation-delay:-120s" alt="">
  <div class="layer mountains" data-depth="0.15"></div>
  <div class="layer hills-far" data-depth="0.25"></div>
  <div class="layer forest" data-depth="0.35"></div>
  <div class="layer hills-near" data-depth="0.5"></div>

  <div class="intro">
    <span class="badge"><b>Version 0.9.1 Beta</b> · Neu: Abenteuer · Open Source · Linux, Windows, macOS</span>
    <h1 class="title">Elora</h1>
    <p class="tagline">Schnelles 2D-Multiplayer: hooken, schwingen, sprengen – mit Freunden im Internet oder im LAN. Und jetzt auch allein im Abenteuer.</p>
    <div class="buttons">
      <a class="btn primary" href="<?= GITHUB ?>/releases">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12m0 0l-5-5m5 5l5-5M4 20h16"/></svg>
        Herunterladen
      </a>
      <a class="btn secondary" href="<?= GITHUB ?>">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 .5a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1.1-.7.1-.7.1-.7 1.2.1 1.9 1.2 1.9 1.2 1.1 1.9 2.9 1.3 3.6 1 .1-.8.4-1.3.8-1.6-2.7-.3-5.5-1.3-5.5-6a4.7 4.7 0 0 1 1.3-3.2c-.2-.3-.6-1.6.1-3.2 0 0 1-.3 3.3 1.2a11.4 11.4 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.7 1.6.3 2.9.1 3.2a4.7 4.7 0 0 1 1.3 3.2c0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 .5Z"/></svg>
        Projekt auf GitHub
      </a>
    </div>
  </div>

  <div class="scene" aria-hidden="true">
    <img src="assets/tree-round.svg" class="deco" style="left:11%;width:150px" alt="">
    <img src="assets/tree-pine.svg" class="deco hide-sm" style="left:20%;width:110px" alt="">
    <img src="assets/bush-2.svg" class="deco" style="left:39%;width:120px" alt="">
    <img src="assets/flower-pink.svg" class="deco" style="left:44%;width:26px" alt="">
    <img src="assets/flower-yellow.svg" class="deco" style="left:57%;width:26px" alt="">
    <img src="assets/grass-1.svg" class="deco" style="left:61%;width:36px" alt="">
    <img src="assets/fence.svg" class="deco hide-sm" style="left:79%;width:90px" alt="">
    <img src="assets/tree-round.svg" class="deco hide-sm" style="left:90%;width:170px" alt="">
    <img src="assets/flower-blue.svg" class="deco" style="left:25%;width:24px" alt="">
    <img src="assets/sign-arrow.svg" class="deco hide-sm" style="left:84%;width:56px" alt="">
    <img src="assets/health.svg" class="pickup" style="left:25%;bottom:70px" alt="">
    <img src="assets/armor.svg" class="pickup hide-sm" style="left:76%;bottom:80px;animation-delay:-.8s" alt="">
    <?= elora('blue', 'left', '1-lachen') ?>
    <?= elora('sun', 'big', '0-herz') ?>
    <?= elora('red', 'right', '6-gg') ?>
  </div>
  <div class="ground" aria-hidden="true"></div>
</header>

<section class="status" id="status">
  <div class="wrap">
    <div class="card">
      <div class="status-head">
        <span class="live"><span class="dot<?= $servers ? '' : ' off' ?>"></span> Live-Status</span>
        <div class="stats">
          <span class="stat"><b><?= count($servers) ?></b>Server online</span>
          <span class="stat"><b><?= $playersOnline ?></b>Spieler im Spiel</span>
        </div>
      </div>
      <?php if (!$servers): ?>
        <div class="empty">
          <?= elora('sun', '', '7-schlaf') ?>
          <p>Gerade ist kein Server online. Starte im Spiel unter <b>„Server erstellen“</b> einen eigenen – mit <b>„Im Internet anzeigen“</b> erscheint er hier und in der Serverliste aller Spieler.</p>
        </div>
      <?php else: ?>
        <div class="servers">
          <?php foreach ($servers as $s): ?>
            <?php $pct = $s['max'] > 0 ? min(100, round($s['clients'] * 100 / $s['max'])) : 0; ?>
            <div class="server">
              <div>
                <div class="name"><?= h($s['name']) ?></div>
                <div class="meta"><span class="chip"><?= h($s['mode']) ?></span> &nbsp;Karte <b><?= h($s['map']) ?></b> · <?= h($s['addr']) ?></div>
              </div>
              <div class="fill" aria-hidden="true"><i style="width:<?= (int) $pct ?>%"></i></div>
              <div class="count"><?= (int) $s['clients'] ?> / <?= (int) $s['max'] ?></div>
              <?php if ($s['players']): ?>
                <div class="players">
                  <?php foreach ($s['players'] as $p): ?>
                    <span class="p <?= h($p['team']) ?>"><?= h($p['name']) ?></span>
                  <?php endforeach; ?>
                </div>
              <?php endif; ?>
            </div>
          <?php endforeach; ?>
        </div>
      <?php endif; ?>
      <p class="note">Stand <?= h($checked) ?> Uhr · aktualisiert sich alle <?= STATUS_TTL ?> Sekunden beim Neuladen. Im Spiel findest du alle Server unter „Spielen“.</p>
    </div>
  </div>
</section>

<section id="abenteuer" style="background:#fff">
  <div class="wrap editor">
    <div>
      <span class="new reveal">Neu in 0.9.1 · Vorschau</span>
      <h2 class="reveal">Das Abenteuer: Die verstummten Quellen</h2>
      <p class="lead reveal">Die Quellen des Taulands verstummen, und die Farben weichen aus dem Dorf Tauwinkel. Elora zieht los – allein, mit Hook, Hammer und einer Menge Mut.</p>
      <div class="chapters">
        <div class="card chapter reveal"><span class="tag" style="background:var(--sun)">P</span><div><b>Tauwinkel</b><p>Oma Pfütze, Tüftel, Klonk, Lotte und Pip – hier lernst du alles, was du brauchst.</p></div></div>
        <div class="card chapter reveal"><span class="tag" style="background:var(--pink)">1</span><div><b>Blütenwiesen</b><p>Verirrte Bienen und eine sehr schlecht gelaunte Brummbär-Hummel.</p></div></div>
        <div class="card chapter reveal"><span class="tag" style="background:var(--green)">2</span><div><b>Murmelwald</b><p>Ein Uhu voller Geschichten, ein Pilzkind auf dem Heimweg und der Wurzelwächter.</p></div></div>
        <div class="card chapter reveal"><span class="tag" style="background:#e0b85a">3</span><div><b>Glutsandwüste</b><p>Sirups Karawane, Treibsand, flirrende Hitze und die Sandschlange.</p></div></div>
      </div>
      <ul class="ticks reveal" style="margin-top:22px">
        <li><b>Neue Fähigkeiten</b> aus jeder Quelle: Hook-Ruck, Heranhooken, Stampfen</li>
        <li><b>Stufen, Fähigkeitenbaum, Ausrüstung</b> und Waffen-Ausbau bei Klonk</li>
        <li><b>Aufgaben und Gespräche</b> – und ein Dorf, das mit jeder Quelle bunter wird</li>
      </ul>
      <p class="reveal" style="color:var(--ink-soft)">Frostspitzen, Sternschlucht und das Finale folgen mit den nächsten Versionen.</p>
    </div>
    <div class="card adv-scene reveal" aria-hidden="true">
      <div class="sun"></div>
      <img src="assets/figur-oma.svg" class="fig" style="left:-12%" alt="">
      <img src="assets/figur-pip.svg" class="fig hide-sm" style="left:8%" alt="">
      <?= elora('sun', 'adv') ?>
      <img src="assets/figur-tueftel.svg" class="fig flip" style="left:36%" alt="">
      <img src="assets/figur-sirup.svg" class="fig flip hide-sm" style="left:56%" alt="">
      <img src="assets/figur-palma.svg" class="fig flip" style="left:70%" alt="">
      <div class="floor"></div>
    </div>
  </div>
</section>

<section id="spiel">
  <div class="wrap">
    <h2 class="reveal">Schnell, direkt, ein bisschen verrückt</h2>
    <p class="lead reveal">Elora spielt sich wie die großen Vorbilder des Genres: flinke Bewegung, ein Haken zum Schwingen und eine Handvoll Waffen, die man in Sekunden versteht und in Wochen meistert. Alles läuft mit 50 Ticks pro Sekunde, flüssig auch bei 100 ms Ping.</p>
    <div class="features">
      <div class="card feature reveal">
        <div class="icon b"><svg viewBox="0 0 52 52"><path d="M8 44 L40 12" stroke="#e8e8e8" stroke-width="5" stroke-linecap="round"/><path d="M8 44 L40 12" stroke="#2b2b2b" stroke-width="1.5" stroke-dasharray="2 4"/><circle cx="40" cy="12" r="7" fill="#e8e8e8" stroke="#2b2b2b" stroke-width="3"/></svg></div>
        <h3>Hook</h3>
        <p>Häng dich an Wände, schwing durch die Karte und zieh Gegner zu dir. Wer den Hook beherrscht, ist überall.</p>
      </div>
      <div class="card feature reveal">
        <div class="icon y"><?= img('hammer') ?></div>
        <h3>Hammer</h3>
        <p>Immer dabei: haut Gegner weg – und dich selbst in die Höhe, wenn du den Boden triffst.</p>
      </div>
      <div class="card feature reveal">
        <div class="icon g"><?= img('grenade') ?></div>
        <h3>Granatwerfer</h3>
        <p>Bogenschüsse um die Ecke, Flächenschaden und Raketensprünge für die ganz schnellen Wege.</p>
      </div>
      <div class="card feature reveal">
        <div class="icon v"><?= img('laser') ?></div>
        <h3>Laser</h3>
        <p>Präzise auf Distanz, prallt einmal von Wänden ab. In Instagib reicht ein Treffer.</p>
      </div>
    </div>

    <div class="modes">
      <div class="card mode reveal"><span class="tag" style="background:var(--green)">DM</span><div><b>Deathmatch</b><br><small>Jeder gegen jeden</small></div></div>
      <div class="card mode reveal"><span class="tag" style="background:var(--blue)">TDM</span><div><b>Team-Deathmatch</b><br><small>Rot gegen Blau</small></div></div>
      <div class="card mode reveal"><span class="tag" style="background:var(--red)">CTF</span><div><b>Capture the Flag</b><br><small>Flagge holen, heimbringen</small></div></div>
      <div class="card mode reveal"><span class="tag" style="background:var(--violet)">LMS</span><div><b>Last Man Standing</b><br><small>Ein Leben pro Runde</small></div></div>
      <div class="card mode reveal"><span class="tag" style="background:var(--pink)">LTS</span><div><b>Last Team Standing</b><br><small>Das Team, das übrig bleibt</small></div></div>
      <div class="card mode reveal"><span class="tag" style="background:#2b2b2b">i</span><div><b>Instagib</b><br><small>Für jeden Modus: nur Laser</small></div></div>
    </div>
  </div>
</section>

<section id="karten" style="background:#fff">
  <div class="wrap">
    <h2 class="reveal">Fünf Karten zum Start</h2>
    <p class="lead reveal">Drei Arenen für Deathmatch und zwei spiegelgleiche Karten für Capture the Flag – mit Plattformen, Eis, Sprungfeldern und Laufbändern.</p>
    <div class="maps">
      <div class="card map m-wiese reveal"><div class="pic"><img src="assets/tree-round.svg" style="left:16%;width:96px" alt=""><img src="assets/bush-1.svg" style="left:60%;width:60px" alt=""><img src="assets/flower-pink.svg" style="left:84%;width:18px" alt=""><div class="floor"></div></div>
        <div class="txt"><h3>Wiese</h3><p>DM · 4–8 Spieler · Sprungfelder zu den Simsen</p></div></div>
      <div class="card map m-wueste reveal"><div class="pic"><img src="assets/rock-2.svg" style="left:14%;width:80px" alt=""><img src="assets/sign-arrow.svg" style="left:68%;width:44px" alt=""><img src="assets/rock-1.svg" style="left:84%;width:30px" alt=""><div class="floor"></div></div>
        <div class="txt"><h3>Wüste</h3><p>DM · 8–12 Spieler · Laufbänder über der Stachelgrube</p></div></div>
      <div class="card map m-winter reveal"><div class="pic"><img src="assets/tree-pine.svg" style="left:14%;width:74px" alt=""><img src="assets/tree-pine.svg" style="left:62%;width:56px" alt=""><img src="assets/fence.svg" style="left:78%;width:50px" alt=""><div class="floor"></div></div>
        <div class="txt"><h3>Winter</h3><p>DM · 12–16 Spieler · Eisseen und Sprungfeld-Schacht</p></div></div>
      <div class="card map m-wald reveal"><div class="pic"><img src="assets/tree-pine.svg" style="left:6%;width:70px" alt=""><img src="assets/tree-round.svg" style="left:34%;width:92px" alt=""><img src="assets/mushroom-red.svg" style="left:76%;width:22px" alt=""><img src="assets/flag.svg" style="left:86%;width:30px" alt=""><div class="floor"></div></div>
        <div class="txt"><h3>Wald</h3><p>CTF · 8–12 Spieler · Festungen und Baumhaus-Route</p></div></div>
      <div class="card map m-nacht reveal"><div class="pic"><img src="assets/moon.svg" style="left:66%;width:60px;bottom:78px;filter:none" alt=""><img src="assets/tree-pine.svg" style="left:10%;width:66px" alt=""><img src="assets/rock-2.svg" style="left:46%;width:56px" alt=""><img src="assets/mushroom-red.svg" style="left:80%;width:22px;filter:drop-shadow(0 0 6px #ff9a9a)" alt=""><div class="floor"></div></div>
        <div class="txt"><h3>Nacht</h3><p>CTF · 12–16 Spieler · Steinburgen und Tunnel</p></div></div>
    </div>
  </div>
</section>

<section id="editor">
  <div class="wrap editor">
    <div>
      <h2 class="reveal">Bau deine eigene Karte</h2>
      <p class="lead reveal">Der Karten-Editor steckt direkt im Spiel. Bauen, auf <b>F5</b> drücken, losspielen – und wieder zurück.</p>
      <ul class="ticks reveal">
        <li><b>Werkzeuge</b> wie Pinsel, Rechteck, Füllen, Auswahl kopieren und einfügen</li>
        <li><b>Materialien</b> mit automatischen Kanten: Erde, Sand, Schnee, Stein und Eis</li>
        <li><b>Deko und Hintergründe</b> mit Parallax – Tag oder Nacht als Vorlage</li>
        <li><b>Animationen</b> für ziehende Wolken, Bäume im Wind oder leuchtende Pilze</li>
        <li><b>Eigene Grafiken</b> als SVG einbetten – Server schicken Karten automatisch an alle Spieler</li>
      </ul>
    </div>
    <div class="card panel reveal" aria-hidden="true">
      <div class="grid"></div>
      <div class="blk" style="left:6%;bottom:30px;width:56%;height:62px"></div>
      <div class="blk" style="left:14%;top:96px;width:22%;height:40px"></div>
      <div class="blk" style="left:42%;top:60px;width:20%;height:34px;background:#566068;box-shadow:none"></div>
      <div class="sel" style="left:11%;top:86px;width:28%;height:62px"></div>
      <img src="assets/tree-round.svg" style="position:absolute;left:8%;bottom:88px;width:70px" alt="">
      <img src="assets/bush-1.svg" style="position:absolute;left:44%;bottom:88px;width:46px" alt="">
      <div class="side"><b>Werkzeuge</b><span>1 Pinsel</span><span>2 Rechteck</span><span>3 Füllen</span><span class="on">5 Auswahl</span><span>8 Deko</span>
        <b>Ebenen</b>Hintergründe · Deko · Spielfläche · Entities<b>Karte</b>„Meine Insel“ · 60 × 30<br>▶ Testspielen (F5)</div>
    </div>
  </div>
</section>

<section id="skins" style="background:#fff;overflow:hidden">
  <div class="wrap" style="text-align:center">
    <h2 class="reveal">Elora und Freunde</h2>
    <p class="lead reveal" style="margin:0 auto">Wähle Farben für Augen, Körper und Füße – und sag es mit Emotes, wenn Worte zu langsam sind.</p>
    <div class="skins reveal">
      <?= elora('green', '', '4-staunen') ?>
      <?= elora('blue', '', '1-lachen') ?>
      <?= elora('sun', '', '0-herz') ?>
      <?= elora('violet', '', '6-gg') ?>
      <?= elora('pink', '', '7-schlaf') ?>
      <?= elora('mint', '', '1-lachen') ?>
    </div>
  </div>
</section>

<section id="open-source">
  <div class="wrap">
    <div class="card oss reveal">
      <?= elora('sun', '') ?>
      <div>
        <h2 style="font-size:30px">Offen für alle</h2>
        <p>Elora ist freie Software: Der Code steht unter der GPL-3.0, eigene Grafiken und Sounds unter CC-BY-SA 4.0. Geschrieben in Rust – Fehler melden, Karten bauen und mitentwickeln ist ausdrücklich erwünscht.</p>
        <div class="badges"><span class="chip">GPL-3.0</span><span class="chip">CC-BY-SA 4.0</span><span class="chip">Rust</span><span class="chip">wgpu</span><span class="chip">Linux · Windows · macOS</span></div>
      </div>
      <a class="btn primary" href="<?= GITHUB ?>">Zum Quellcode</a>
    </div>
  </div>
</section>

<footer>
  <img src="assets/moon.svg" class="moon" alt="">
  <div class="wrap row">
    <div>
      <div class="logo">Elora</div>
      <p>Schnelles 2D-Multiplayer, inspiriert von den Klassikern des Genres.</p>
    </div>
    <div style="text-align:right">
      <p><a href="<?= GITHUB ?>">github.com/ehrenberg/elora</a></p>
      <p><a href="<?= GITHUB ?>/releases">Downloads</a> · <a href="<?= GITHUB ?>/issues">Fehler melden</a></p>
      <p>Code GPL-3.0 · Grafiken CC-BY-SA 4.0 · Schrift Inter (OFL)</p>
    </div>
  </div>
</footer>

<script>
// Parallax der Hintergrund-Ebenen beim Scrollen und leicht mit der Maus
(() => {
  const layers = [...document.querySelectorAll('.layer')];
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  let mx = 0, raf = 0;
  const draw = () => {
    raf = 0;
    const y = scrollY;
    for (const l of layers) {
      const d = parseFloat(l.dataset.depth);
      l.style.transform = `translate3d(${mx * d * -40}px, ${y * d * .6}px, 0)`;
    }
  };
  const ask = () => { if (!raf) raf = requestAnimationFrame(draw); };
  addEventListener('scroll', ask, { passive: true });
  addEventListener('pointermove', e => { mx = e.clientX / innerWidth - .5; ask(); }, { passive: true });
  // Abschnitte beim Hineinscrollen einblenden
  const io = new IntersectionObserver(es => es.forEach(e => {
    if (e.isIntersecting) { e.target.classList.add('shown'); io.unobserve(e.target); }
  }), { threshold: .12 });
  document.querySelectorAll('.reveal').forEach(el => io.observe(el));
})();
// ohne JavaScript bleibt alles sichtbar
document.documentElement.classList.add('js');
</script>
<noscript><style>.reveal { opacity: 1; transform: none; }</style></noscript>
</body>
</html>
