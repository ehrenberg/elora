# Quest order audit (E-379)

Status: 2026-11-12 · scope: chapters 1–5 (prologue `well`, main quests `blossom_spring` to
`sternschlucht`, `finale`, all side quests) in `assets/adventure/quests.toml`,
`assets/adventure/dialogs/*.toml`, `crates/elora-adventure/src/{quest,state}.rs`, `session/`.

## Trigger

Owner's playtest on 2026-11-12: the Bumblebear (chapter 1 guardian) was calmed while
`blossom_spring` was still at an earlier step (Wabe not yet asked). The `guardian` step only
listened for the defeat *event*; the guardian stays defeated (E-235), so the event could never
come again and the quest was stuck.

## The class of bug

Goals were checked in two ways: `collect` and `flag` against the save game (state), `talk`,
`bring`, `reach` and `defeat` only as events while their step was active. An event that happens
before its step is lost. That is harmless when it can be repeated (talk to someone again, walk
into a map again, defeat another beetle) and a soft-lock when it cannot (a guardian, a one-shot
flag, a dialog branch that no longer opens, loot that is gone).

## Fix in the engine

- `SaveGame::goal_met` (state check, used by `update_quests` after every change, so also the
  moment a step becomes active):
  - `collect`: items held (as before);
  - `flag`: value **at least** `value` (was: exactly – a counter overshooting would have stuck);
  - `reach` map: `visited:<map>` (set on every map entry since E-264);
  - `reach` zone: new flag `reached:<map>:<zone>`, set by `on_reach` only for zones that are a
    quest goal (keeps the save game small);
  - `defeat` of a guardian (`boss = true`): `defeated.<kind>`; now also set by
    `SaveGame::on_defeat`, not only by the session;
  - `talk`, `bring`, ordinary enemies, `manual`: events (repeatable, see the check below).
- **Save game repair** (`SaveGame::repair_quests`, run by `Session::new`): old stuck save games
  complete every step whose goal is already met; if a `bring` step needs a key item that a
  defeated guardian of an earlier step drops (the spring spark) and it is missing, it comes back.
- **Guardian key loot** (`Session::guardian_keys`): the spring spark no longer lies around as
  loot after the fight but goes straight into the bag – the guardian never returns, so a spark
  left behind (death, falling, leaving) was a soft-lock of its own.
- **Content check** (`check::story` → `quest_order`): a guardian `defeat` goal needs `count = 1`;
  every character of a `talk`/`bring` goal needs a dialog with an entry without condition, so the
  step can always be finished by talking again.
- **Test** `every_quest_step_counts_when_done_out_of_order` (`tests/story.rs`): for every step
  with a state goal, the goal happens first (quest not even started), then the step is made
  active – it must be passed at once. Covers all 30+ such steps of chapters 1–5.

## Findings

| # | Where | Finding | Fix |
|---|---|---|---|
| 1 | `blossom_spring.guardian`, `murmelwald.guardian`, `glutsand.guardian`, `frostspitzen.guardian` | Guardian defeated before the step was active → step waits forever (the playtest bug) | Engine: `defeated.<kind>` counts; load repair. Tests `bumblebear_defeated_before_its_step_still_counts`, `a_stuck_save_game_repairs_itself_on_load` |
| 2 | all `reach` steps (`deeper`, `roots`, `spring`, `west_slope`, `ruins`, `departure`, `ridge`, `heights`, …) | Map entered before the step → had to walk back in (annoying, and with #1 the chain up to the arena) | Engine: `visited:<map>` counts |
| 3 | `well.meadow_edge`, `ruin.chamber` (zone goals) | Zone entered before the step → had to walk in again | Engine: `reached:<map>:<zone>` |
| 4 | Guardian loot (`spring_spark` from all four guardians) | Spark not picked up after the fight → `spark` step impossible, guardian gone | Spark goes straight into the bag; load repair restores a missing one |
| 5 | `dialogs/tueftel.toml` spark entries | Tüftel handed out the ability, `freed.*` flags and the party *without* the spark – quest stayed at `spark` | Entries need `has spring_spark` (test `tueftel_needs_the_spark_for_his_reward`) |
| 6 | `dialogs/klonk.toml` crystals entry | Offered only while `frostspitzen` was active – visiting Klonk after chapter 4 meant the side quest could never start (crystals collected for nothing) | Condition `not quest frostspitzen new` (test `klonk_offers_the_crystals_after_chapter_four_too`) |
| 7 | `dialogs/luma.toml`, `luma_song.sing` | A talk goal is used up by whichever node opens: with eight fragments in the bag the `fragments` entry won, `song_complete` never ran and `flag luma_song` (stronger song in the climax) was never set | Song entry before the fragments; the climax also counts `quest luma_song done` |
| 8 | Flag goals with a counter (`climbers.found`, `oasis.watered`, `luma_song.verses`) | Checked with `==`; an overshoot would stick (none possible today, but fragile) | `>=` as documented |

## Checked and fine (no change)

- **Talk goals** (`well.tueftel/klonk`, `wabe`, `plumm`, `sirup`, `flocke`, `luma`, parties,
  `tablet`, `report`, `thanks`): the characters stay and always open a dialog; the step-specific
  entries (`advice`, `practice` with the hammer, `party*`, `report`, `thanks`) are first or cannot
  collide with another quest's entry at that time. Klonk sends Elora to Tüftel first
  (`first_tueftel`), so the prologue order is kept by the dialog.
- **Bring goals**: Flocke's rope taken from the chest before talking to Flocke → first talk opens
  the greeting (step `flocke` done), the second hands the rope over. Light threads, bees, runes,
  fragments, crystals, glitter stone collected before their quest → the `collect` step passes
  when the quest starts, the next talk hands them over.
- **Side quests started late**: climbers found before Flocke asks (`climbers.found` counts
  anyway), oasis (water only exists after Palma's waterskin), tunes (only count while
  `luma_song` is active and are not used up before), mushroom child (`mushroom_child.home` is a
  state flag), buried chamber (zone flag, #3). All offers stay open (`quest <x> new` choices).
- **Climax** (`sternschlucht.protect`): `luma_protected` is a state flag; the climax starts only
  after Luma's ruins dialog (`luma.sings`) and never twice.
- **Guardians twice or never**: a defeated guardian is remembered per map object and never
  respawns; one that was left mid-fight returns at full health. Victory screen and `defeated.*`
  flags only on the actual defeat.
- **Oma's spark node** (`quest <x> active and flag defeated.<guardian>`): with #1 the quest
  catches up on the next change, so her hint and the quest book agree again.

## Open / not covered

- Map reachability (could a player reach chapter N+1's guardian while chapter N's spark is still
  undelivered, so two sparks mix?) is assumed from the ability gates (hook jerk, pull hook,
  stomp, ice grip, star cape); there is no automatic reachability check.
- A talk goal is still used up by any dialog of the character (#7 is fixed in the content; the
  rule is in the handbook). A general fix would bind talk goals to entries – not needed today.
