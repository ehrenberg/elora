# Quellen der Assets

Eigene Assets (Figur, Items, Emotes, prozedurale Sounds) sind im Projekt entstanden. Fremde
Assets stehen hier mit Quelle und Lizenz (E-081, E-107). Nur **CC0** (gemeinfrei) oder mit
dem Projekt verträgliche Lizenzen.

## Sounds (`assets/sounds/files/`)

Alle Dateien per `cargo xtask sound-import` umgewandelt (Mono, 44,1 kHz, 16 Bit, Stille am
Anfang entfernt, Ende ausgeblendet, Spitze −1 dBFS, teils gekürzt).

| Sound | Originaldatei | Paket | Autor | Lizenz |
|---|---|---|---|---|
| `air_jump` | `cloth3.ogg` | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `chat` | `pluck_002.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `emote` | `drop_002.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `flag_capture` | `jingles_STEEL07.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `flag_drop` | `jingles_PIZZI16.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `flag_grab_enemy` | `jingles_PIZZI04.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `flag_grab_own` | `jingles_PIZZI10.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `flag_return` | `jingles_STEEL09.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `grenade_explode` | `explosionCrunch_000.ogg` + `lowFrequency_explosion_000.ogg` (0–1,6 s, gemischt 1 : 0,9, Nachhall ausgeblendet) | [Kenney – sci-fi-sounds](https://kenney.nl/assets/sci-fi-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `grenade_fire` | `impactSoft_heavy_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `hammer_fire` | `cloth3.ogg` (Tonhöhe ×0,6, Tiefpass 1,8 kHz, rückwärts anschwellend + kurzer Ausklang) | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `hammer_hit` | `impactPunch_medium_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `hit_confirm` | `tick_002.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `hook_attach_ground` | `impactWood_light_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `hook_attach_player` | `impactSoft_medium_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `hook_fire` | `drawKnife1.ogg` | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `hook_no_attach` | `impactMetal_light_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `jump` | `slime_000.ogg` (Tonhöhe ×1,8, 0,15 s) | [Kenney – sci-fi-sounds](https://kenney.nl/assets/sci-fi-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `land` | `footstep_carpet_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `laser_bounce` | `glass_002.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `laser_fire` | `glass_004.ogg` (Tonhöhe ×0,7, kurzes Echo, 0,3 s) | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `no_ammo` | `metalClick.ogg` | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `pain_long` | `slime_001.ogg` | [Kenney – sci-fi-sounds](https://kenney.nl/assets/sci-fi-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `pain_short` | `slime_000.ogg` | [Kenney – sci-fi-sounds](https://kenney.nl/assets/sci-fi-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `pickup_armor` | `metalLatch.ogg` | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `pickup_health` | `pluck_001.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `pickup_respawn` | `bong_001.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ui_click` | `drop_003.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ui_select` | `select_006.ogg` (0–0,15 s) | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ui_open` | `maximize_008.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ui_close` | `minimize_008.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `voice` | `drop_002.ogg` (0–0,09 s, Plapperlaut mit Tonhöhe je Figur, E-286) | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `boss_land` | `impactSoft_heavy_002.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `deflect` | `impactGlass_light_001.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `collect` | `confirmation_001.ogg` | [Kenney – interface-sounds](https://kenney.nl/assets/interface-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `fanfare` | `jingles_STEEL01.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `quest_done` | `jingles_PIZZI01.ogg` | [Kenney – music-jingles](https://kenney.nl/assets/music-jingles) | Kenney (kenney.nl) | CC0 1.0 |
| `root_emerge` | `creak3.ogg` | [Kenney – rpg-audio](https://kenney.nl/assets/rpg-audio) | Kenney (kenney.nl) | CC0 1.0 |
| `root_strike` | `impactWood_heavy_000.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `sand_dig` | `footstep_snow_002.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `shell_clack` | `impactPlate_light_001.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ice_crack` | `impactGlass_light_002.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `ice_break` | `impactGlass_heavy_001.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `icicle_shatter` | `impactGlass_medium_003.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |
| `snow_crunch` | `footstep_snow_001.ogg` | [Kenney – impact-sounds](https://kenney.nl/assets/impact-sounds) | Kenney (kenney.nl) | CC0 1.0 |

## Wetterklänge (`assets/ambience/`)

Per ffmpeg nach Ogg Vorbis umgewandelt (Stereo, 44,1 kHz, Qualität 4), auf etwa −20 LUFS
gebracht; Schleifen mit weich überblendeter Nahtstelle (E-338).

| Datei | Original | Quelle | Autor | Lizenz |
|---|---|---|---|---|
| `regen.ogg` | `1.ogg` aus „Rain OGG.zip“ | [OpenGameArt – Rain (loopable)](https://opengameart.org/content/rain-loopable) | Ylmir | CC0 1.0 |
| `wind.ogg` | `low-rumbling-176033.mp3` | [OpenGameArt – Low Rumbling](https://opengameart.org/content/low-rumbling) | Musheran | CC0 1.0 |
| `sand.ogg` | `wind background noise 2.wav` (Hochpass 700 Hz, Höhen +8 dB) | [OpenGameArt – Mild Wind Background Noise](https://opengameart.org/content/mild-wind-background-noise) | Bashar3A | CC0 1.0 |
| `donner.ogg` | `rain-thunder.ogg` (20,5–38,5 s, ausgeblendet) | [OpenGameArt – Rain + Long Thunder](https://opengameart.org/content/rain-long-thunder) | WuxiaScrub | CC0 1.0 |
| `feuer.ogg` | `fire.wav` (Schleife, Nahtstelle überblendet) | [OpenGameArt – Fireplace Sound loop](https://opengameart.org/content/fireplace-sound-loop) | PagDev | CC0 1.0 |

## Schriften (`assets/fonts/`)

Inter und JetBrains Mono unter SIL Open Font License 1.1, siehe `assets/fonts/*-OFL.txt` und `THIRD_PARTY_LICENSES`.

## Musik (`assets/music/`)

Per ffmpeg nach Ogg Vorbis umgewandelt (Stereo, 44,1 kHz, Qualität 4); das Spiel entpackt beim Abspielen.

| Datei | Titel | Quelle | Autor | Lizenz |
|---|---|---|---|---|
| `menu.ogg` | „FM fun“ | [OpenGameArt – FM fun](https://opengameart.org/content/fm-fun) | sla97 | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) |
| `tauwinkel.ogg` | „Heavenly Loop“ | [OpenGameArt – Heavenly Loop](https://opengameart.org/content/heavenly-loop) | isaiah658 | CC0 1.0 |
| `bluetenwiesen.ogg` | „Sunset Plains“ | [OpenGameArt – Sunset Plains](https://opengameart.org/content/sunset-plains) | yoiyami | CC0 1.0 |
| `boss.ogg` | „Urban Boss Battle“ | [OpenGameArt – Urban Boss Battle](https://opengameart.org/content/urban-boss-battle) | mintodog | CC0 1.0 |
| `fest.ogg` | „Medieval: Minstrel Dance“ (Loop-Fassung) | [OpenGameArt – Minstrel Dance](https://opengameart.org/content/medieval-minstrel-dance) | randommind | CC0 1.0 |
| `murmelwald.ogg` | „Woodland Fantasy“ | [OpenGameArt – Woodland Fantasy](https://opengameart.org/content/woodland-fantasy) | Matthew Pablo | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) |
| `boss-wald.ogg` | „Bamboo Blitz“ | [OpenGameArt – Bamboo Blitz](https://opengameart.org/content/bamboo-blitz) | Tsorthan Grove | CC0 1.0 |
| `wueste.ogg` | „Desert Loop“ | [OpenGameArt – Desert Loop](https://opengameart.org/content/desert-loop) | iamoneabe | CC0 1.0 |
| `boss-wueste.ogg` | „Hard Boss Battle 1“ | [OpenGameArt – Hard Boss Battle 1](https://opengameart.org/content/hard-boss-battle-1) | MintoDog | CC0 1.0 |
| `frost.ogg` | „Ice Village“ | [OpenGameArt – Ice Village](https://opengameart.org/content/ice-village) | KarateStudios | CC0 1.0 |
| `boss-frost.ogg` | „Dramatic Boss Encounter“ | [OpenGameArt – Dramatic Boss Encounter](https://opengameart.org/content/dramatic-boss-encounter) | cynicmusic | CC0 1.0 |
