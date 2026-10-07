# Adventure progression system (O-204)

Status: **decided** (E-241 to E-245, A1.3) · Framework: E-206, E-212, E-214/E-215, E-219, E-220, E-239, [world book §6](world-book.md)

Principle: hook, jump and double jump stay as they are (E-212). **Progression never changes running speed, jump height or hook pull**, but health, damage, the five area abilities, loot and comfort.

All numbers are starting values (code **P-xx**) and will later live in data files under `assets/adventure/`.

## 1. Levels and experience

| # | Value | Proposal |
|---|---|---|
| P-01 | Maximum level | **30** (end of the main story around level 20–22, then side content and spring trials) |
| P-02 | Experience to the next level | **15 + 10 × level** (level 1→2: 25, 10→11: 115, 29→30: 305; 4785 in total up to level 30) |
| P-03 | Experience from enemies | from `creatures.toml` (currently 5–8), bosses 100–300 |
| P-04 | Experience from quests | side quests 30–120, main quests 80–250 |
| P-05 | Per level | **1 dewdrop point**; every **2 levels +1 health** (10 → 24 at level 30, E-241) |
| P-06 | Level-up | refills health, short display with effect |

## 2. Skill tree (at Tüftel's)

Three branches; each node has 1–3 ranks at 1 point each. A node unlocks when the one above it has at least rank 1. **35 ranks in total** (the proposal mistakenly said 41); up to level 30 there are 29 points plus 2 from special quests, i.e. 31 – not everything (E-242): the choice matters. Movement nodes require the corresponding area ability.

| Branch | Node (ranks) | Effect per rank |
|---|---|---|
| **Movement** | Quick jerk (3) | Hook jerk cooldown −150 ms |
| | Long hook (2) | Hook length +5 % |
| | Heavy stomp (3) | Shock wave +16 radius, +1 damage |
| | Firm grip (2) | Cling duration +0.4 s |
| | Far glide (2) | Fall while gliding −0.4 |
| **Combat** | Strength (3) | Damage of all weapons +10 % |
| | Quick hand (3) | Fire delay −8 % |
| | Impact (2) | Knockback on enemies +25 % |
| | Ammo pouch (2) | Ammo +2 |
| | Stunning hammer (1) | Hammer stuns enemies for 0.5 s |
| **Spring** | More health (3) | +1 health |
| | Drop magnet (2) | Loot magnet +48 |
| | Gleam finder (2) | +10 % gleam drops |
| | Long shelter (2) | Protection after a hit +250 ms |
| | Healing blossoms (2) | Healing plants heal +1 |
| | Second chance (1) | Once per map, continue with 3 health instead of dying |

## 3. Weapons and upgrades (at Klonk's)

| # | Topic | Proposal |
|---|---|---|
| P-10 | Weapons in the adventure | Start with the **hammer**; **grenade launcher** at the end of chapter 1 (reward from Klonk), **laser** at the end of chapter 3 |
| P-11 | Ammo | **as in multiplayer, via pickups and chests** (E-243) |
| P-12 | Upgrade levels | **3 levels** per weapon (I–III) |
| P-13 | Cost | Gleam drops + material from the matching area, e.g. hammer I: 40 + 3 amber, II: 120 + 4 resin, III: 300 + 3 star shards |

| Weapon | Level I | Level II | Level III |
|---|---|---|---|
| Hammer | Impact: +1 damage | Range +30 % | Shock wave: hits everything around Elora |
| Grenade launcher | Explosion +20 % | Shrapnel: 3 small follow-up explosions | +2 ammo |
| Laser | +1 bounce | Pierce: hits up to 3 enemies | Charge time −25 % |

Materials per area: amber (Blütenwiesen), resin (Murmelwald), ember stone (Glutsandwüste), ice crystal (Frostspitzen), star shard (Sternschlucht).

## 4. Equipment and inventory

| # | Topic | Proposal |
|---|---|---|
| P-20 | Slots | Hat, cloak, boots, pendant (E-206) |
| P-21 | Bonuses | 1–2 small bonuses per piece from: health, armour, damage %, loot magnet, gleam drops %, protection time, ability values (as in the tree). **No speed or jump bonuses** |
| P-22 | Rarity | common (1 bonus), rare (2 bonuses), guardian find (2 bonuses + special trait) |
| P-23 | Armour | only from equipment (E-239); refilled at save points |
| P-24 | Inventory | **no slot limit**; tabs equipment, consumables, materials, key items (keys, quest items – not sellable) |
| P-25 | Consumables | Healing potion (+5 health), dew potion (hook jerk without cooldown, 10 s); at most 5 per kind |
| P-26 | Selling | at Lotte's for 40 % of the purchase price |

## 5. Death and saving

| # | Topic | Proposal |
|---|---|---|
| P-30 | Loss on death (E-220) | **25 %** of the gleam drops collected since the last save point (rounded down); everything else is kept |
| P-31 | Save points | Spring stone: saves, refills health and armour |
| P-32 | Save game | 3 slots (E-219) in the user directory (`saves/platz-1.esav` …), **compressed with checksum, not readable** (E-245), with format version; a broken slot is reported, never overwritten |
| P-33 | Content | Level, experience, health, gleam drops, tree, inventory, equipment, weapon levels, abilities, quests, world state (switches, doors, crumbling floor, chests, defeated bosses, consequences of conversations), location (map + save point), play time |
