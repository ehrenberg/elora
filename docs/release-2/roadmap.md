# Roadmap Release 2

Status: **order set (E-216)** · Focus areas: E-202

## Goals

1. **Players & community:** leftovers from Release 1 (credits, playtests, balancing), demos and replays, spectator camera, remote console.
2. **More game content:** more weapons, new maps and themes, sounds in maps, music.
3. **Teammate bots:** computer opponents that really play along – for training, for full servers with few people, and as the basis for enemies and NPCs in the adventure.
4. **Role-playing adventure:** single player with a story in a **hub world** (village + unlockable areas, E-203); levels & skill tree, weapon upgrades, equipment & loot, quests & dialogues (E-206). Plus a **role-playing PvP mode** in which you level up during the match (E-204). Story: Claude proposes, the project owner decides (E-205).

## Dependencies

```
Adventure foundation ─► Story & adventure ─► Role-playing PvP mode
        │  (abilities, enemies, progression, save games, dialogues, quests, editor)
        └─► Teammate bots (use enemy control and pathfinding)
Weapons & content, community run in between
```

- **Adventure foundation first** (E-216): technology and editor tools before story and maps are produced in volume.
- **Bots afterwards:** the multiplayer bots build on the control and pathfinding of the adventure enemies; bots deliver inputs like players (E-035).
- **Game feel stays core:** role-playing stats must not dilute the movement and hook feel; values such as damage or health in the adventure are separate tuning sets.
- **Release 1 leftovers** (playtests, balancing, packages on Windows/macOS) are handled by the project owner on the side (O-51).

## Milestones

| # | Milestone | Content |
|---|---|---|
| R2-M1 | Adventure foundation | Abilities, enemies, progression, save games, NPCs, dialogues, quests, editor; acceptance with the prologue – plan: [`a1-plan.md`](a1-plan.md) |
| R2-M2 | Story & adventure | Tauwinkel, five areas, bosses, finale, side quests ([`world-book.md`](world-book.md)); sub-milestones per chapter: M2.1 Blütenwiesen ([`m2-1-plan.md`](m2-1-plan.md)), M2.2 Murmelwald ([`m2-2-plan.md`](m2-2-plan.md)), M2.3 Glutsandwüste ([`m2-3-plan.md`](m2-3-plan.md)), M2.4 Frostspitzen ([`m2-4-plan.md`](m2-4-plan.md), accepted E-349), M2.5 Sternschlucht, M2.6 finale |
| R2-W1 ✅ | Weather | Rain, thunderstorm, fog, wind, sandstorm, snow in the adventure and as a map property; light gameplay effect in the adventure (E-329 to E-335) – **accepted** (E-339) – plan: [`w1-plan.md`](w1-plan.md) |
| R2-RF | Refactoring | Before the next release: English throughout (E-348) and cleanup of large files, duplicated logic and lint allows – plan: [`refactoring-plan.md`](refactoring-plan.md) |
| R2-M3 | Teammate bots | Pathfinding, combat and hook behaviour, difficulty levels, bots on servers and in training |
| R2-M4 | Weapons & content | More weapons, sounds in maps, new themes and maps |
| R2-M5 | Role-playing PvP mode | “Quellenkampf”: levels, upgrades and loot during the match (E-204) |
| R2-M6 | Community | Demos and replays, spectator camera, remote console |
| R2-M7 | Release 2 | Acceptance, packages, publication |

## To be clarified (together)

See open points O-200 to O-207 in [`decisions.md`](decisions.md).

Idea for the time after Release 2: a **persistent world** on a server (O-208).
