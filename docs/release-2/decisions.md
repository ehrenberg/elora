# Decision log – Release 2

Continued from **E-200**. Release 1 (E-001 to E-173) is in the
[archive](../archive/release-1/02-decisions.md), the principles that still apply are in
[`../handbook/principles.md`](../handbook/principles.md).

## Decisions made

| # | Date | Topic | Decision | Reason / source |
|---|---|---|---|---|
| E-200 | 2026-10-02 | Documentation | **Archive + handbook:** Release 1 moves to `docs/archive/release-1/`, knowledge that still applies becomes a handbook in `docs/handbook/`, Release 2 in `docs/release-2/` | Project owner decision |
| E-201 | 2026-10-02 | Decision log | Release 1 log archived, **new log from E-200**; principles summarised in the handbook | Project owner decision |
| E-202 | 2026-10-02 | Release 2 focus areas | **Players & community, more game content, teammate bots** and a **role-playing adventure**: single player with story, NPCs and role-playing elements (levelling, upgrading weapons and abilities …), also as a game mode | Project owner decision |
| E-203 | 2026-10-02 | World structure (O-200) | **Hub with areas:** a village as a meeting point with NPCs, traders and quests; areas (e.g. forest, desert, ice mountains, caves) are unlocked step by step | Project owner decision |
| E-204 | 2026-10-02 | Role-playing as a game mode (O-201) | **Role-playing PvP mode:** a separate game mode in which you level up and upgrade weapons/abilities during the match (progress applies to the round). No co-op adventure planned | Project owner decision |
| E-205 | 2026-10-02 | Story (O-203) | **Claude proposes** (world, characters, plot in variants), **the project owner decides** | Project owner decision |
| E-206 | 2026-10-02 | Role-playing elements (O-204) | **All four:** levels & skills (skill tree), weapon upgrades, equipment & loot (inventory, traders, currency), quests & dialogues | Project owner decision |
| E-207 | 2026-10-02 | Story (O-203) | **Draft A “The Silent Springs”** from [`story-drafts.md`](story-drafts.md): fairy-tale, warm; village Tauwinkel, five areas (Blütenwiesen, Murmelwald, Glutsandwüste, Frostspitzen, Sternschlucht), antagonist “the Withered One”, reconciliation instead of victory; PvP mode “Quellenkampf” | Project owner decision |
| E-208 | 2026-10-02 | Credits: name | Project owner as **Bastian Ehrenberg** | Project owner decision |
| E-209 | 2026-10-02 | Credits: location (M8.5) | **Last page in the settings** (“About Elora”) | Project owner decision |
| E-210 | 2026-10-02 | Story twist | **Sixth spring beneath the village well**, the Withered One as its forgotten guardian – fits | Project owner decision |
| E-211 | 2026-10-02 | Age group | The adventure is **always playable for 12+**: fights yes, but without blood and cruelty; the guardian fights may be real fights, the story tells them as calming | Project owner decision |
| E-212 | 2026-10-02 | Names, abilities | The names of the world and characters as well as the five abilities in their order (hook jerk, pull hook, stomp, ice grip, glide) **stay** | Project owner decision |
| E-213 | 2026-10-02 | Dialogues | **Choices with consequences** allowed | Project owner decision |
| E-214 | 2026-10-02 | Play time | The single-player mode should **last a long time – several hours** (target values see [world book §8](world-book.md)) | Project owner decision |
| E-215 | 2026-10-02 | Play time: target values | **Main story 6–8 h, 12 h+ with side content**, about 20 adventure maps, ~25 side quests, collectibles, “spring trials” after the ending ([world book §8](world-book.md)) | Project owner decision |
| E-216 | 2026-10-02 | Release 2 order | **Adventure foundation first** (R2-M1, plan: [`a1-plan.md`](a1-plan.md)); the multiplayer bots later reuse its enemy control and pathfinding | Project owner decision |
| E-217 | 2026-10-02 | Adventure language (D-A1-01) | **German and English from the start** | Project owner decision |
| E-218 | 2026-10-02 | Maintaining conversations and quests (D-A1-02) | **Text files**, visible in the editor (which NPC has which conversation) and directly testable | Project owner decision |
| E-219 | 2026-10-02 | Save games (D-A1-03, O-202) | **3 slots**, automatic on map change and at save points; no saving in the middle of a fight | Project owner decision |
| E-220 | 2026-10-02 | Death in the adventure (D-A1-04) | **Back to the last save point with a small loss:** part of the gleam drops collected since then is lost; items and experience are kept | Project owner decision |
| E-221 | 2026-10-02 | Difficulty (D-A1-05) | **Only one difficulty level** | Project owner decision |
| E-222 | 2026-10-02 | Presentation of conversations (D-A1-06) | **Both:** conversations in the text box at the bottom with a picture of the character and choices; short call-outs as a speech bubble above the character | Project owner decision |
| E-223 | 2026-10-02 | Abilities in multiplayer (D-A1-07) | New abilities **only in the adventure and in Quellenkampf**; the other modes stay unchanged | Project owner decision |
| E-224 | 2026-10-02 | Camera in the adventure (D-A1-08) | **Slightly look-ahead**, plus **camera zones** from the map (e.g. boss rooms); mouse look as usual | Project owner decision |
| E-225 | 2026-10-02 | Designs A1.0 | **Characters, objects and UI accepted** ([`design/`](design/)); the adventure menu additionally gets a **map** | Project owner decision |
| E-226 | 2026-10-02 | Hook jerk (A1.1) | New key **“Ability”** (freely bindable): jerk towards the hook point while the hook is attached to a wall, with cooldown | Project owner decision |
| E-227 | 2026-10-02 | Stomp (A1.1) | Press **down in the air** | Project owner decision |
| E-228 | 2026-10-02 | Ice grip (A1.1) | Clinging only on a **dedicated climbing tile**; it is a **solid, non-hookable** wall | Project owner decision |
| E-229 | 2026-10-02 | Glide (A1.1) | **Hold jump while falling** (after the double jump, or once it is used up) | Project owner decision |
| E-230 | 2026-10-02 | Crumbling floor | Breaks on a stomp and **stays broken**; remembered in the save game in the adventure, until the end of the round in PvP | Project owner decision |
| E-231 | 2026-10-02 | Acceptance A1.1 | Abilities **accepted** in the playtest; A-01 to A-10 are starting values | Project owner decision |
| E-232 | 2026-10-02 | Jumping on enemies (A1.2) | **No effect:** enemies are only hit by weapons and stomp; contact hurts Elora | Project owner decision |
| E-233 | 2026-10-02 | Hook and enemies (A1.2) | The hook **grabs enemies and pulls Elora towards them**; with pull hook small enemies are pulled to Elora | Project owner decision |
| E-234 | 2026-10-02 | Protection after a hit (A1.2) | **Briefly invulnerable (blinking) and knockback** away from the enemy | Project owner decision |
| E-235 | 2026-10-02 | Enemy respawn (A1.2) | **When re-entering the map**; bosses and special enemies stay defeated | Project owner decision |
| E-236 | 2026-10-02 | Loot (A1.2) | **Pops out, Elora attracts it from nearby** (small magnet); loot on the ground does not disappear | Project owner decision |
| E-237 | 2026-10-02 | Self-damage in the adventure | **No self-damage**, only knockback (grenade jumps stay free) | Project owner decision |
| E-238 | 2026-10-02 | Enemy health | **Small bar after a hit** for a few seconds; bosses with a large bar at the top | Project owner decision |
| E-239 | 2026-10-02 | Elora's health in the adventure | **10 as in multiplayer**, raised later by levels and equipment; armour only via equipment | Project owner decision |
| E-240 | 2026-10-02 | Acceptance A1.2 | Enemies **accepted** in the playtest; values in `creatures.toml` and A-03, A-11 to A-15 are starting values | Project owner decision |
| E-241 | 2026-10-02 | Levels (O-204) | Maximum level 30, experience to the next level 15 + 10 × level, 1 dewdrop point per level; **+1 health every 2 levels** (10 → 24) | Project owner decision |
| E-242 | 2026-10-02 | Skill tree (O-204) | Three branches, 16 nodes, **not everything reachable** (recounted: 35 ranks, 31 points up to level 30) ([`progression.md`](progression.md) §2) | Project owner decision |
| E-243 | 2026-10-02 | Weapons in the adventure (O-204) | Start with the hammer, grenade launcher after chapter 1, laser after chapter 3; **ammo as in multiplayer, only via pickups and chests**; 3 upgrade levels per weapon with gleam drops and area material | Project owner decision |
| E-244 | 2026-10-02 | Equipment, inventory, death (O-204) | As proposed: no slot limit, no speed or jump bonuses, loss on death 25 % of the gleam drops collected since saving | Project owner decision |
| E-245 | 2026-10-02 | Save games (O-202) | **No readable files:** save games are stored compressed with a checksum | Project owner decision |
| E-246 | 2026-10-02 | Conversation format (A1.4) | **TOML like the other data** | Project owner decision |
| E-247 | 2026-10-02 | Translations (A1.4) | **Both languages side by side** in the same file; the check reports missing translations | Project owner decision |
| E-248 | 2026-10-02 | Consequences of choices (E-213) | **Affection per character** (unlocks extras), **rewards**, **flags in the world state**, **starting and completing quests** | Project owner decision |
| E-249 | 2026-10-02 | Quest steps (A1.4) | Talk to, reach a place, defeat enemies, collect, deliver, trigger in the world | Project owner decision |
| E-250 | 2026-10-02 | Failing quests | **Some quests can fail** (through choices or conditions) | Project owner decision |
| E-251 | 2026-10-02 | Quest log | Shows **completed steps and the current one**, further ones as “?” | Project owner decision |
| E-252 | 2026-10-02 | Map transitions (A1.5) | **Selectable per transition:** open paths when walking in, doors, caves and gates with the action key | Project owner decision |
| E-253 | 2026-10-02 | Action key | **E is the action key everywhere** (talk, open, use); the **emote wheel is on Ctrl everywhere** | Project owner decision |
| E-254 | 2026-10-02 | Doors and gates | **Solid wall like stone** (not hookable), opens as soon as a condition holds and stays open | Project owner decision |
| E-255 | 2026-10-02 | Chests | **Fixed content per chest**, once opened it stays open; optionally locked (key or condition) | Project owner decision |
| E-256 | 2026-10-02 | Switches | **Lever with the action key, toggleable** (flag 1/0), optionally only once; also switches triggered with the hammer or by pull hook | Project owner decision |
| E-257 | 2026-10-02 | NPCs | **Stand, turn towards Elora**, shout call-outs; some with a short walking route | Project owner decision |
| E-258 | 2026-10-02 | Healing plants | **Touching heals** (base value 2 health, more with “Healing blossoms”), **grow back when re-entering the map** | Project owner decision |
| E-259 | 2026-10-02 | Camera zones | **Lock** (fixed view, e.g. boss room) **or limit** (camera stays within the area), smooth transition | Project owner decision |
| E-260 | 2026-10-02 | Leaving the adventure (A1.6) | Progress **counts up to the last save** (map change, spring stone); the pause menu warns | Project owner decision |
| E-261 | 2026-10-02 | Death in the adventure (A1.6) | **Separate screen with choices:** “Continue at the spring stone” or “Main menu” (loss as per E-220) | Project owner decision |
| E-262 | 2026-10-02 | Storage of adventure maps | **Separate folder `maps/abenteuer/`**, not in server and vote lists | Project owner decision |
| E-263 | 2026-10-02 | Adventure menu (A1.7) | Opens with **Tab** (there is no scoreboard in the adventure), the game is paused meanwhile | Project owner decision |
| E-264 | 2026-10-02 | “Map” tab | **World map of the Tauland:** Tauwinkel and the five areas, unlocked ones in colour, Elora's location, sections of the current area | Project owner decision |
| E-265 | 2026-10-02 | Healing potions | **Hotkey Q** drinks a healing potion, other consumables via the inventory | Project owner decision |
| E-266 | 2026-10-02 | Look-ahead camera (E-224) | **Slightly in the running direction** (up to about 3 tiles, smooth), plus mouse look and camera zones | Project owner decision |
| E-267 | 2026-10-02 | Camera when standing still (playtest A1.7) | **Look-ahead stays:** offset in the last running direction, switching only after running briefly in the other direction (replaces the pull-back from E-266) | Project owner decision |
| E-268 | 2026-10-02 | Editor tool (A1.8) | **One tool “Adventure” (key 9):** choose the kind in the sidebar, click places on the grid, areas by dragging; click selects, drag moves, Del deletes, undo as usual | Project owner decision |
| E-269 | 2026-10-02 | Test play in the editor (A1.8) | **Selectable test state:** level, abilities, weapons, flags; start at the chosen entrance or at the mouse; no save game is changed, transitions load other maps along | Project owner decision |
| E-270 | 2026-10-02 | Conversations in the editor (E-218) | **Preview and test window:** entries, nodes, answers and conditions per NPC; play through a conversation in the window with the test state, see changes to flags, quests and affection; show errors, reload files without restarting | Project owner decision |
| E-271 | 2026-10-02 | Location of adventure maps | **User folder `maps/abenteuer/`**, takes precedence over the shipped map of the same name; into the project after approval | Project owner decision |
| E-272 | 2026-10-02 | Prologue flow (A1.9) | **As in the draft** [`prolog.md`](prolog.md): Pip wakes Elora, Oma at the well, practice with Tüftel and Klonk, Lotte, Pip's side quest, first section of the Blütenwiesen up to the spring stone | Project owner decision |
| E-273 | 2026-10-02 | Explaining the controls | **Signposts** that you read with E | Project owner decision |
| E-274 | 2026-10-02 | Buildings in Tauwinkel | **Designs for selection first** (houses, well, workshop, smithy, shop, treehouse), then the map | Project owner decision |
| E-275 | 2026-10-02 | Paleness of the village (E-210) | ~~Desaturated look of the map~~ (replaced by E-277): via a colour filter (characters stay colourful), gets weaker with every freed spring (flag) | Project owner decision |
| E-276 | 2026-10-02 | Look of the buildings (designs A1.9) | **No drop-shaped roofs**, not too childish: child-friendly but **to be taken seriously** (proper roofs, half-timbering, stone, wood) | Project owner decision |
| E-277 | 2026-10-02 | Paleness of the village (replaces E-275) | **No colour filter:** the village stays drawn normally, **individual things are faded** (flowers, flags, flowerbeds, well square) and get their colour back with every freed spring (decoration variants via flag) | Project owner decision |
| E-278 | 2026-10-02 | Buildings of Tauwinkel | **Second version accepted** (`design/tauwinkel-buildings.png`) | Project owner decision |
| E-279 | 2026-10-02 | Map edges (playtest A1.9) | **Open where the way continues:** the path runs out of the picture, the transition spans the full height; **where the world ends, a natural boundary** (steep slope, rocks, dense forest), no walls | Project owner decision |
| E-280 | 2026-10-02 | Size of Tauwinkel (playtest A1.9) | **About 450 tiles wide**, houses and characters with large gaps, longer paths between places | Project owner decision |
| E-281 | 2026-10-02 | Practice ground (playtest A1.9) | **Tüftel's yard in the village**, larger and winding: hooking on the ceiling, swinging over gaps, stone (hook slips off), crumbling floor, a reward at the top | Project owner decision |
| E-282 | 2026-10-02 | Blütenwiesen (playtest A1.9) | **Designed for movement:** height differences (cliffs, hollows, overhangs), hook chasms, several routes (upper harder with rewards, lower easier); maps fuller overall | Project owner decision |
| E-283 | 2026-10-02 | Thorn pits | **Falling in costs health and puts Elora back at the edge**, no death (adventure) | Project owner decision |
| E-284 | 2026-10-02 | Acceptance R2-M1 (A1.10) | **Prologue accepted** (“that fits”), R2-M1 completed | Project owner decision |
| E-285 | 2026-10-02 | Source of new sounds and music (extends E-109) | **CC0 from the web:** clicks from Kenney “interface-sounds”, music from CC0 collections; music is presented for listening first; sources in `assets/SOURCES.md` | Project owner decision |
| E-286 | 2026-10-02 | Speech sounds | **Babble sounds:** short, soft syllable sounds while the text appears, pitch per character | Project owner decision |
| E-287 | 2026-10-02 | Weapon on pickup | **Setting with three levels:** off / only new weapons / always; default: only new weapons | Project owner decision |
| E-288 | 2026-10-02 | Smearing during fast movement (playtest) | Description: **soft streaks** (movement fluid, image blurry); narrow down the cause and try countermeasures | Project owner feedback |
| E-289 | 2026-10-02 | Menu music | **“FM fun” by sla97** ([OpenGameArt](https://opengameart.org/content/fm-fun)) in the main menu; licence **CC BY 4.0** (credit in `SOURCES.md` and “About”), exception to E-285 on request | Project owner decision |
| E-290 | 2026-10-02 | Music per area | **Tauwinkel: “Heavenly Loop”** (isaiah658), **Blütenwiesen: “Sunset Plains”** (yoiyami), both CC0 from OpenGameArt; field `music` per area in `worldmap.toml` | Project owner decision |
| E-291 | 2026-10-02 | Time of day in the main menu | **4 phases by system clock:** morning (6–10 h), day (10–17), evening (17–21, lanterns on), night (21–6, stars, moon, glowing windows); smooth transitions | Project owner decision |
| E-292 | 2026-10-02 | More playful menu background | **Existing graphics:** village scene (houses, well, trees, props), characters (Elora in her skin, Pip, Oma, Tüftel, Klonk, Lotte; some walking), animals and motion (butterflies, birds, smoke, clouds, flags), enemies (spike beetles, grass hoppers) | Project owner decision |
| E-293 | 2026-10-02 | Training map | **About 120 × 40** with all new features (climbing walls, crumbling floor, thorns, special tiles, enemy practice area) and playful decoration | Project owner decision |
| E-294 | 2026-10-02 | Stutter in online play (feedback) | “Everything seems to have a certain jitter/stutter” – find and fix the causes | Project owner feedback |
| E-295 | 2026-10-02 | AI services for content | **None** (OpenAI, openart.ai, Recraft rejected): graphics continue to be made with the Python scripts in `tools/design/`, music and sounds from free sources (E-285) | Project owner decision |
| E-296 | 2026-10-02 | Splitting R2-M2 | **Sub-milestones per chapter:** M2.1 Blütenwiesen, M2.2 Murmelwald, M2.3 Glutsandwüste, M2.4 Frostspitzen, M2.5 Sternschlucht, M2.6 finale; each part plan → designs → implementation → acceptance | Project owner decision |
| E-297 | 2026-10-02 | Chapter 1 sections | **Three new ones:** `wiese-2` (apiary, giant flowers, stream), `wiese-3` (caves beneath the roots), `wiese-arena` (Blossom Spring) | Project owner decision |
| E-298 | 2026-10-02 | Bumblebear fight | **Harder:** circling with pollen, dive with warning, dazed on the ground; from half health faster, two dives, hook blossoms wilt alternately; from one third she calls confused bees that fly back to Wabe after the fight | Project owner decision |
| E-299 | 2026-10-02 | Bumblebee vulnerability | **Only when dazed on the ground**, in the air hits bounce off | Project owner decision |
| E-300 | 2026-10-02 | Reward “Wabe's Bees” | **Honeycomb hat** (equipment with a small bonus) and experience | Project owner decision |
| E-301 | 2026-10-02 | Village after chapter 1 | **Small celebration:** conversation at the well, lanterns, garlands and festive music until Elora moves on; plus more colour and new call-outs | Project owner decision |
| E-302 | 2026-10-02 | Explaining hook jerk | **Practice ground in the yard:** Tüftel explains and opens a new part of his yard with a course that only works with hook jerk | Project owner decision |
| E-303 | 2026-10-03 | Chapter 1 designs (M2.1.0) | **Accepted** as presented ([`design/chapter1-drafts.png`](design/chapter1-drafts.png)) | Project owner decision |
| E-304 | 2026-10-04 | Chapter 1 music | **Fight: “Urban Boss Battle”** (mintodog), **celebration: “Minstrel Dance”** (randommind), both CC0 from OpenGameArt | Project owner decision |
| E-305 | 2026-10-05 | Acceptance R2-M2.1 | **Accepted for now** (“leave it like this for now”) after the fixes from the playtest (hook jerk immediate, icon sizes, map list); continue with R2-M2.2 | Project owner decision |
| E-306 | 2026-10-05 | Path into the Murmelwald | **Path up the western slope** of Tauwinkel, the forest begins at the top | Project owner decision |
| E-307 | 2026-10-05 | Root Warden fight | **As proposed:** roots from the ground with warning, pulling out cores makes him vulnerable, from half health root walls, faster at the end | Project owner decision |
| E-308 | 2026-10-05 | Mushroom child | **Follows Elora** as a companion, waits at difficult spots | Project owner decision |
| E-309 | 2026-10-05 | Memory runes reward | **Piece of equipment and a dewdrop point** | Project owner decision |
| E-310 | 2026-10-05 | Celebration after chapter 2 | **As after chapter 1** | Project owner decision |
| E-311 | 2026-10-05 | Mushroom imp effect | **Colourful trip:** Elora rainbow-coloured and slower for a few seconds, the world wobbles slightly; child-friendly | Project owner decision |
| E-312 | 2026-10-05 | Chapter 2 designs (M2.2.0) | **Accepted** ([`design/chapter2-drafts.png`](design/chapter2-drafts.png)); the Root Warden as a walking tree giant with a bark face and moss beard (inspired by Treebeard) | Project owner decision |
| E-313 | 2026-10-05 | Chapter 2 music | **Murmelwald: “Woodland Fantasy”** (Matthew Pablo, CC BY 3.0, credit in `SOURCES.md` and “About”), **Root Warden: “Bamboo Blitz”** (Tsorthan Grove, CC0) | Project owner decision |
| E-314 | 2026-10-05 | Acceptance R2-M2.2 | **Accepted** (“fits like this”); continue with R2-M2.3 Glutsandwüste | Project owner decision |
| E-315 | 2026-10-05 | Path into the Glutsandwüste | **Branch off the east path:** sunken path south down into the desert | Project owner decision |
| E-316 | 2026-10-05 | Sand Serpent and stomp | **Hammer and grenades hit** the surfaced serpent; stomp (after the fight) later does double damage | Project owner decision |
| E-317 | 2026-10-05 | Sand crab | **Only vulnerable from above**, from the side hits bounce off | Project owner decision |
| E-318 | 2026-10-05 | Quicksand | **Sinks in slowly, slows down, jumping frees you;** sunk in deep: small damage and back to the edge | Project owner decision |
| E-319 | 2026-10-05 | Oasis quest | Fill a **water skin** in the ruins, water three withered patches of the oasis | Project owner decision |
| E-320 | 2026-10-05 | Heat in the desert | **Heat shimmer and heat bar:** sun fills it, shade and oasis cool down, full = Elora gets slower | Project owner decision |
| E-321 | 2026-10-05 | Chapter 3 designs (M2.3.0) | **Accepted** ([`design/chapter3-drafts.png`](design/chapter3-drafts.png)); heat shimmer as a shader (post-processing of the world), not as a drawn overlay | Project owner decision |
| E-322 | 2026-10-06 | Water skin | **Three fillings:** fill once at the ruin spring, enough for all three withered patches | Project owner decision |
| E-323 | 2026-10-06 | Ruin side quest | Given by **Sirup**; in the buried chamber (with stomp) a stone tablet and as reward the **sun veil** (hat: heat bar fills more slowly) | Project owner decision |
| E-324 | 2026-10-06 | Sirup's shop | **Small shop with rare goods** (consumable, material, a trinket) | Project owner decision |
| E-325 | 2026-10-06 | The grey wanderer in chapter 3 | **Traces and reports:** grey footprints, ruin tablets, Sirup saw a grey figure drinking at the oasis at night; no appearance | Project owner decision |
| E-326 | 2026-10-06 | Chapter 3 music | **Glutsandwüste: “Desert Loop”** (iamoneabe, CC0), **Sand Serpent: “Hard Boss Battle 1”** (MintoDog, CC0) | Project owner decision |
| E-327 | 2026-10-06 | Acceptance R2-M2.3 | **Chapter 3 accepted for now**; implemented from the playtest: hook jerk noticeably faster, Klonk hands out weapons first, ammo chests, Sand Serpent much harder, victory screen after every guardian | Project owner decision |
| E-328 | 2026-10-06 | Graphical improvements | **Implement:** lively motion (squash, stretch, bob, breathing), flight poses rotate with the movement, soft shadows under characters and enemies, colour returns with the springs (saturation as a shader); later separate backgrounds per area and light in dark areas | Project owner decision |
| E-329 | 2026-10-06 | Weather: scope | **Adventure and maps:** per area in the adventure, plus as a map property in the editor (multiplayer maps too, everyone sees the same) | Project owner decision |
| E-330 | 2026-10-06 | Weather: gameplay effect | **Light effect in the adventure** (wind pushes, wet or snowy ground slightly slippery, fog shortens the view); visuals only in multiplayer | Project owner decision |
| E-331 | 2026-10-06 | Weather: control | **Story and chance:** overcast as long as an area's spring is silent; afterwards, on entering, random between fair weather and the area's weather types | Project owner decision |
| E-332 | 2026-10-06 | Times of day | **Later, weather first** (O-209 stays open for the times of day) | Project owner decision |
| E-333 | 2026-10-06 | Weather types | **Rain, thunderstorm, fog, wind (leaves, blossoms), sandstorm, snow and blizzard** | Project owner decision |
| E-334 | 2026-10-06 | Weather sounds | **Free recordings (CC0)**, presented for listening | Project owner decision |
| E-335 | 2026-10-06 | Weather setting | **Full, gentle, off** under graphics; the gameplay effect in the adventure stays the same | Project owner decision |
| E-336 | 2026-10-06 | Plan R2-W1 | **Approved** with the proposals D-W1-01 to D-W1-03; **lightning can do damage** (only in the adventure, with a short warning on the ground) | Project owner decision |
| E-337 | 2026-10-06 | Weather designs (W1.0) | **Accepted** ([`design/weather-drafts.png`](design/weather-drafts.png)) | Project owner decision |
| E-338 | 2026-10-06 | Weather sounds (W1.5) | After a listening test: **rain** “Rain (loopable)” no. 1 (Ylmir), **wind** “Low Rumbling” (Musheran), **thunder** “Rain + Long Thunder” (WuxiaScrub, excerpt), **sand** from “Mild Wind Background Noise” (Bashar3A) filtered brighter; all CC0 (OpenGameArt) | Project owner decision |
| E-339 | 2026-10-06 | Acceptance R2-W1 | **Weather accepted**: nine weather types with visuals, sound and a light gameplay effect in the adventure (wind, wetness, lightning with warning, visibility); release maps `dm-winter`, `ctf-nacht`, `dm-wueste` with weather; R2-W1 completed | Project owner decision |
| E-340 | 2026-10-06 | Chapter 4: ice grip | **In the middle of the chapter:** Flocke gives climbing claws (= ice grip) in the mountain village; the guardian fight uses the walls; the spring spark strengthens the ice grip | Project owner decision |
| E-341 | 2026-10-06 | Chapter 4: Kristella | **Frost waves and walls:** hovers above an ice hall, freezes the floor in waves (fresh frost hurts), Elora escapes onto climbing walls; exhausted, she sinks down and is vulnerable; later icicles | Project owner decision |
| E-342 | 2026-10-06 | Chapter 4: cold | **Cold bar** mirroring the heat: it fills outdoors, faster in a blizzard; fireplaces and huts warm; full = slower | Project owner decision |
| E-343 | 2026-10-06 | Chapter 4: terrain | **Icicles, avalanches, thin ice**; no jump pads | Project owner decision |
| E-344 | 2026-10-06 | Plan R2-M2.4 | **Approved** with the proposals D-M24-01 to D-M24-10 ([`m2-4-plan.md`](m2-4-plan.md)) | Project owner decision |
| E-345 | 2026-10-06 | Chapter 4 designs (M2.4.0) | **Accepted** ([`design/chapter4-drafts.png`](design/chapter4-drafts.png)) | Project owner decision |
| E-346 | 2026-10-07 | Chapter 4 texts (M2.4.5) | **Approved** ([`m2-4-content.md`](m2-4-content.md)); victory screen: „Kristella lächelt wieder – und die Frostquelle glitzert klar wie ein Wintermorgen.“, honorary title „Gipfelstürmerin“ | Project owner decision |
| E-347 | 2026-10-07 | Chapter 4 music (M2.4.8) | After a listening test: **Frostspitzen** “Ice Village” (KarateStudios, CC0), **Kristella** “Dramatic Boss Encounter” (cynicmusic, CC0); sounds from Kenney packs (CC0) and procedural, fireplace “Fireplace Sound loop” (PagDev, CC0) | Project owner decision |
| E-348 | 2026-10-07 | Language policy and refactoring | **From now on English** for README, code identifiers, code comments and documentation; player-facing text stays translated in `assets/lang`. Before the next release a refactoring pass ([`refactoring-plan.md`](refactoring-plan.md)) converts existing code and docs and improves structure. Nothing is converted yet. | Project owner decision |
| E-349 | 2026-10-07 | Acceptance R2-M2.4 | **Chapter 4 accepted** after the playtest fixes (Kristella harder, thin ice, icicles, confetti, weather particles, no shot on map change) | Project owner decision |
| E-350 | 2026-10-07 | Refactoring decisions | D-RF-01 **content ids in English** (with save-game migration); D-RF-02 **condition language in English** (German keywords accepted for one release); D-RF-03 **archive is translated too**; D-RF-04 **start after the chapter 4 acceptance** (now) | Project owner decision |
| E-351 | 2026-10-07 | First-start setup screen | **One page** with name, language and Elora's look (colors with preview); **mandatory on the first start** (no settings file yet), no skipping; built **as part of the refactoring** (R2-RF, task RF-30), directly in English and in the new structure | Project owner decision |
| E-352 | 2026-10-08 | Text language in code (RF-06) | Error messages that **players see** get keys in `assets/lang` (de/en), technical detail is appended; **logs, server console, CLI help and xtask output in English only**; client option `--abenteuer` becomes **`--adventure` without alias** |
| E-353 | 2026-10-08 | Weather before a spring is freed (playtest) | Gloomy weather becomes **mostly dry**: about 55% dry and grey, 35% rain, 10% thunderstorm (or the area's own gloomy kind); after freeing unchanged |
| E-354 | 2026-10-08 | Guidance for things that do not exist yet (playtest) | Signposts for abilities Elora does not have yet show **a different text** (come back later) and explain the ability only once she has it; path signs to the Glutsandwüste and Frostspitzen say the way is **blocked, with a hint** what is needed; all hints and conversations are checked for things that do not exist yet; signposts about **twice as high** |
| E-355 | 2026-10-08 | Intro video for the adventure (playtest) | A **real video**, skippable (Esc/Space), at the start of a new adventure. **Exception to E-295:** the owner creates the video with an external AI tool (licence must allow redistribution); E-295 stays in force otherwise. Playback with a **built-in video decoder** in the client |
| E-356 | 2026-10-08 | Video decoder for the intro (E-355) | **AV1 via `rav1d`** (BSD-2, pure Rust); `unsafe` is allowed **only** in a small separate crate `elora-video` that wraps its C-style API (exception to the workspace rule); everything else keeps `unsafe_code` denied |

## Open points

Carried over from Release 1:

- [ ] **O-45 Remote console** (admin commands from the client with a password)
- [ ] **O-49 More distribution channels** (itch.io, Flathub, Steam, own website)
- [ ] **O-50 macOS on Intel** (cross-build or own runner)
- [ ] **O-51 M8 leftovers:** ~~credits page~~ (done, E-209); playtests and balancing (M8.6) and tests of the packages on Windows/macOS are done later by the project owner
- [ ] **O-52 Demos and replays** (E-115: after Release 1)

New for Release 2 (role-playing adventure, see [`roadmap.md`](roadmap.md)):

- [x] ~~O-200 World structure~~ → E-203
- [x] ~~O-201 Role-playing as a game mode~~ → E-204
- [x] ~~O-202 Saving progress~~ → E-219 (in PvP mode only for the round, E-204)
- [x] ~~O-203 Story and world~~ → E-207 · elaboration: [`world-book.md`](world-book.md) (draft)
- [x] ~~O-204 Progression system~~ → E-241 to E-244 ([`progression.md`](progression.md))
- [ ] **O-205 NPCs and enemies** – behaviour, dialogues, traders, companions; based on the bots
- [ ] **O-206 Adventure in the editor** – NPCs, triggers, dialogues and quests in maps
- [ ] **O-207 More weapons** – which ones, for which modes

Recorded for the future:

- [ ] **O-208 Persistent world** – a server with an ongoing world on which player characters keep their progress (small online RPG; needs accounts and server-side storage) – after Release 2
- [ ] **O-209 Times of day and weather** (weather implemented and accepted with R2-W1, E-329 to E-339; times of day stay open, E-332) – time of day (morning, day, evening, night) and weather (rain, wind, fog, snow, thunderstorm) in the adventure and maps: sky, light, decoration tinting and particles; in the adventure possibly with a gameplay effect (enemies only at night, glowing mushrooms, slippery ground in rain). Basis: times of day in the main menu (E-291), day/night templates in the editor – open, after R2-M2.1
