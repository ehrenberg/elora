# Tuning proposal for Elora

Status: **accepted** (E-023, 2026-09-25) – all values T-01 to T-30 count as starting values; T-31 to T-36 since E-140.

Original values come from [`../archive/release-1/01-teeworlds-analysis.md`](../archive/release-1/01-teeworlds-analysis.md). Units: 1 tile = 32 units, values apply per tick.

## Guiding idea

E-015 calls for a deviation, but only a “slight” one. Therefore:

- **Most values change by at most ±5–10 %.** Larger changes immediately feel alien.
- **The core mechanics stay untouched** (air friction, velocity ramp, ground friction). They shape the feel the most.
- **Elora gets her own character:** a bit **nimbler on the ground**, a **stronger first jump**, in exchange a **weaker double jump**, and a **slightly longer, faster hook** with **less pull**. The result: more speed and momentum without increasing the total jump height.
- All values go into `elora-sim/src/tuning.rs` and can be changed live in the sandbox. We fix the final values after playtesting.

## A. Simulation rate

| # | Value | Original | Proposal | Reason | Decision |
|---|---|---|---|---|---|
| T-01 | Ticks per second | 50 | **50** (unchanged) | All values are defined per tick. With a different rate we would have to convert everything, and the feel would shift uncontrollably. | ✅ |

## B. Movement

| # | Value | Original | Proposal | Δ | Reason | Decision |
|---|---|---|---|---|---|---|
| T-02 | GroundControlSpeed | 10.0 | **10.5** | +5 % | 525 instead of 500 units/s (≈ 16.4 tiles/s). Elora feels a bit nimbler. | ✅ |
| T-03 | GroundControlAccel | 2.0 | **2.2** | +10 % | Elora reaches top speed faster, the controls respond more directly. | ✅ |
| T-04 | GroundFriction | 0.5 | **0.5** | – | Core value for braking, stays. | ✅ |
| T-05 | GroundJumpImpulse | 13.2 | **13.6** | +3 % | Jump height ≈ 185 instead of 174 units (5.8 instead of 5.4 tiles). | ✅ |
| T-06 | AirJumpImpulse | 12.0 | **11.5** | −4 % | Double jump ≈ 132 instead of 144 units. **The total height stays almost the same** (≈ 317 instead of 318), but the first jump counts more. | ✅ |
| T-07 | AirControlSpeed | 5.0 | **5.0** | – | Air control is core feel. | ✅ |
| T-08 | AirControlAccel | 1.5 | **1.6** | +7 % | Minimally more steering in the air | ✅ |
| T-09 | AirFriction | 0.95 | **0.95** | – | Core value, stays | ✅ |
| T-10 | Gravity | 0.5 | **0.5** | – | Affects every trajectory. We control jump height via the impulses instead. | ✅ |
| T-11 | VelrampStart / Range / Curvature | 550 / 2000 / 1.4 | **unchanged** | – | The speed cap stays. | ✅ |

## C. Hook

| # | Value | Original | Proposal | Δ | Reason | Decision |
|---|---|---|---|---|---|---|
| T-12 | HookLength | 380 | **400** | +5 % | 12.5 instead of 11.9 tiles. Elora reaches minimally further. | ✅ |
| T-13 | HookFireSpeed | 80 | **85** | +6 % | Compensates T-12: despite the larger range, the hook takes as long to reach maximum length as in the original (≈ 4.7 instead of 4.75 ticks). At short distance it hits slightly faster. | ✅ |
| T-14 | HookDragAccel | 3.0 | **3.0** | – | Stays | ✅ |
| T-15 | HookDragSpeed | 15.0 | **14.0** | −7 % | Compensates the larger range; the hook stays a tool rather than a catapult. | ✅ |
| T-16 | Player hook duration | 60 ticks (1.2 s) | **55 ticks (1.1 s)** | −8 % | Slightly shorter control over opponents makes the game less frustrating. | ✅ |
| T-17 | Player hook force factor | 1.5 | **1.5** | – | Stays | ✅ |

> **Note on T-12/T-13 (finding from M1.1):** The *effective* wall range results from the flight steps (see analysis §5): Elora **382** (42 + 4·85), original **362** (42 + 4·80) – i.e. +5.5 %. Players are hit up to the full length (400).

## D. Weapons (E-016: hammer, laser, grenade)

| # | Value | Original | Proposal | Reason | Decision |
|---|---|---|---|---|---|
| T-18 | Hammer damage / fire rate | 3 / 125 ms | **3 / 125 ms** | Without the pistol the hammer is the only basic weapon. It should stay reliable. | ✅ |
| T-19 | Hammer knockback | 10 (+1.1 upward component) | **11** | Slightly stronger; the hammer gains more character as a mobility tool. | ✅ |
| T-20 | Laser damage / fire rate | 5 / 800 ms | **5 / 750 ms** | Slightly higher cadence to compensate for the missing pistol and shotgun | ✅ |
| T-21 | Laser range | 800 | **850** | Slightly larger, matching the longer hook | ✅ |
| T-22 | Laser bounces / delay | 1 / 150 ms | **1 / 150 ms** | Stays | ✅ |
| T-23 | Grenade damage / fire rate | 6 / 500 ms | **6 / 500 ms** | Stays, balance anchor | ✅ |
| T-24 | Grenade speed / curvature / lifetime | 1000 / 7.0 / 2 s | **1050 / 7.0 / 2 s** | Slightly faster, more direct trajectory | ✅ |
| T-25 | Explosion radius / inner / MaxForce | 135 / 48 / 12 | **135 / 48 / 12.5** | Rocket jumps get minimally stronger, which fits Elora's weaker double jump. | ✅ |
| T-26 | Self-damage | max(1, Dmg/2) | **max(1, Dmg/2)** | Stays | ✅ |
| T-27 | Max. ammo | 10 | **10** | Stays | ✅ |

## E. Health & pickups

| # | Value | Original | Proposal | Reason | Decision |
|---|---|---|---|---|---|
| T-28 | Max. HP / armor | 10 / 10 | **10 / 10** | Determines time-to-kill, which is core feel. | ✅ |
| T-29 | Pickup respawn | 15 s | **15 s** | Stays | ✅ |
| T-30 | Respawn delay after death | ≈ 0.5 s | **0.5 s** | Stays, fast game | ✅ |

## E2. Tile kinds (M6.1, E-137, E-140)

| # | Value | Original | Proposal | Reason | Decision |
|---|---|---|---|---|---|
| T-31 | Ground friction on ice | – (ground 0.5) | **0.985** | The character slides far after letting go | ✅ |
| T-32 | Acceleration on ice | – (ground 2.0) | **0.35** | Speeding up and braking take noticeably longer | ✅ |
| T-33 | Jump pad force | – | **20** units/tick | Throws about 12 tiles high, clearly more than a jump | ✅ |
| T-34 | Jump pad directions | – | **up, diagonal left, diagonal right (45°)** | Enough for Release 1 | ✅ |
| T-35 | Conveyor speed | – | **4.0** units/tick | Noticeably carries you, you can still walk against it | ✅ |
| T-36 | Platform | – | **passable from below/the sides**; hook, grenade, laser fly through | Like one-way platforms in other games | ✅ |

## E3. Abilities and enemies in the adventure (R2-M1, E-226 to E-239)

Only in the adventure and in the spring fight (E-223). Starting values for playtesting; the sandbox shows them under „Fähigkeiten & Gegner (A-01 bis A-15)“ (abilities & enemies). The values of the enemy kinds are in `assets/adventure/creatures.toml`.

| # | Value | Proposal | Reason | Decision |
|---|---|---|---|---|
| A-01 | Hook jerk speed | **26** units/tick (previously 16) | Well above the normal hook pull (T-15: 14); 16 was barely noticeable in the game | Playtest 2026-10-06 |
| A-02 | Hook jerk cooldown | **800 ms** | One jerk per swing, no rapid fire | ✅ starting value (E-231) |
| A-03 | Pull hook force | **2.5** units/tick² | Small enemies fly in within about half a second | ✅ starting value (E-240) |
| A-04 | Stomp speed | **22** units/tick | Clearly faster than free fall | ✅ starting value (E-231) |
| A-05 | Shockwave radius | **64** (2 tiles) | Breaks the crumbling floor below and next to Elora | ✅ starting value (E-231) |
| A-06 | Ice grip cling duration | **1.0 s** | Hold on briefly, then Elora slides off; ground or a wall jump resets it | ✅ starting value (E-231) |
| A-07 | Slide speed while clinging | **1.0** units/tick | Slow sliding down | ✅ starting value (E-231) |
| A-08 | Wall jump sideways / up | **9 / 12** units/tick | Slightly weaker than the ground jump (T-05: 13.6); the double jump remains | ✅ starting value (E-231) |
| A-09 | Max. falling speed while gliding | **2.0** units/tick | About one fifth of free fall | ✅ starting value (E-231) |
| A-10 | Air control while gliding | **7.0** (normal T-07: 5.0) | Drift far | ✅ starting value (E-231) |
| A-11 | Protection after a hit | **1000 ms** | Blinking, no multiple hits (E-234) | ✅ starting value (E-240) |
| A-12 | Knockback on contact | **8** units/tick (sideways, plus 60 % upwards) | Elora is noticeably pushed away | ✅ starting value (E-240) |
| A-13 | Stomp shockwave damage | **3** | Like a hammer hit | ✅ starting value (E-240) |
| A-14 | Stun from stomp | **1500 ms** | Time for one or two hits | ✅ starting value (E-240) |
| A-15 | Loot magnet | **96** (3 tiles) | Loot flies to Elora from nearby (E-236) | ✅ starting value (E-240) |
| A-16 | Hammer range | **14** (as in multiplayer) | Upgrade “range” (+30 %) | Upgrade (E-243) |
| A-17 | Hammer stuns | **0 ms** | Node “stunning hammer” (500 ms) | Upgrade (E-242) |
| A-18 | Knockback on enemies | **× 1.0** | Node “force” (+25 % per rank) | Upgrade (E-242) |
| A-19 | Hammer shockwave | **off** | Upgrade hammer III: hits all enemies around Elora | Upgrade (E-243) |
| A-20 | Grenade shrapnel | **0** | Upgrade grenade launcher II: 3 small follow-up explosions (⅓ damage, enemies only) | Upgrade (E-243) |
| A-21 | Laser pierce | **0** | Upgrade laser II: hits up to 3 enemies | Upgrade (E-243) |
| A-22 | Thorn damage | **2 health** | Death tiles in the adventure: damage, back to the last safe ground | E-283 |
| A-23 | Speed in the colorful daze | **× 0.55** | Pilzwicht (E-311): this much slower while the daze lasts | E-311 |
| A-24 | Sinking in quicksand | **0.22** units/tick (in whole units) | Slow; a jump frees you | E-318 |
| A-25 | Speed in quicksand | **× 0.4** | Much slower | E-318 |
| A-26 | Damage when fully sunk | **1** | Then back to the edge | E-318 |
| A-27 | Speed with a full heat bar | **× 0.7** | Until the bar drops below half | E-320 |
| A-28 | Hook jerk duration | **320 ms** | This long the jerk pulls straight to the hook point at A-01 instead of pushing only once | Playtest 2026-10-06 |
| A-29 | Wind gust on Elora in the air | **0.15** units/tick at wind 1 | Only in the adventure; on the ground the wind does not push | R2-W1, E-330 |
| A-30 | Lightning strike damage | **2** | Only in the adventure's thunderstorm | E-336 |
| A-31 | Lightning strike radius | **56** units | Around the glowing spot | E-336 |
| A-32 | Warning time before the strike | **900 ms** | Ground glows, sparks rise – time to dodge | E-336 |
| A-33 | Interval between lightning strikes | **9000 ms** at full strength (± 40 %) | Less frequent in a weaker thunderstorm | E-336 |
| A-34 | Wind deflection of grenades | **2.2** | Grenades drift with the wind | R2-W1, E-330 |
| A-35 | Sliding on wet ground | **0.33** (fraction towards ice) | Rain, snow: softer braking | R2-W1, E-330 |
| A-36 | Thin ice holds | **600 ms** | Then it breaks under Elora (cracks as a warning); a stomp breaks it immediately | R2-M2.4, E-343 |
| A-37 | Thin ice grows back | **5500 ms** (playtest, previously 4000) | Only when nobody is stuck in the hole | R2-M2.4 |
| A-38 | Ice water damage | **1** | Then back to the edge | D-M24-05 |
| A-39 | Snowballs per avalanche | **7** | Roll downhill from the higher end | D-M24-06 |
| A-40 | Interval between snowballs | **350 ms** | | D-M24-06 |
| A-41 | Rest after an avalanche | **8000 ms** | Before the same slope goes off again | D-M24-06 |
| A-42 | Pulling up on the climbing wall | **0** (strengthened **1.6** units/tick) | After Tüftel's work with the spring spark of the frost spring (flag `eisgriff.stark`): pressing towards the wall pulls Elora up, ice grip cling duration A-06 doubled | D-M24-03 |

## F. Rules from follow-up decisions

- **Starting equipment (E-025):** spawn with hammer only. Laser and grenade come from pickups with full ammo (10).
- **Instagib (E-026):** laser only, infinite ammo, 1 hit = death, no pickups.
