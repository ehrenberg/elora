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

## Schriften (`assets/fonts/`)

Inter und JetBrains Mono unter SIL Open Font License 1.1, siehe `assets/fonts/*-OFL.txt` und `THIRD_PARTY_LICENSES`.

## Musik (`assets/music/`)

Per ffmpeg nach Ogg Vorbis umgewandelt (Stereo, 44,1 kHz, Qualität 4); das Spiel entpackt beim Abspielen.

| Datei | Titel | Quelle | Autor | Lizenz |
|---|---|---|---|---|
| `menu.ogg` | „FM fun“ | [OpenGameArt – FM fun](https://opengameart.org/content/fm-fun) | sla97 | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) |
| `tauwinkel.ogg` | „Heavenly Loop“ | [OpenGameArt – Heavenly Loop](https://opengameart.org/content/heavenly-loop) | isaiah658 | CC0 1.0 |
| `bluetenwiesen.ogg` | „Sunset Plains“ | [OpenGameArt – Sunset Plains](https://opengameart.org/content/sunset-plains) | yoiyami | CC0 1.0 |
