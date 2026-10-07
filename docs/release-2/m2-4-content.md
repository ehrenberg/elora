# R2-M2.4 – Chapter 4 content (draft, M2.4.5)

Status: **Approved and built in** (texts approved 2026-10-07, E-346) · Basis: [`m2-4-plan.md`](m2-4-plan.md), [`world-book.md`](world-book.md) §4.4 and §5, E-340 to E-345, D-M24-01 to D-M24-10

German texts were approved; the English ones were added when building them in (quoted below as in the game). Conditions and
effects use the notation of the content files ([`../handbook/adventure-content.md`](../handbook/adventure-content.md)).

## 1. Characters

| Id | Name | Where | Voice | Nature |
|---|---|---|---|---|
| `flocke` | Flocke the mountain guide | hut in the mountain village (`frost-2`) | clear, strong (1.05) | energetic, a bit stubborn, laughs loudly; has wanted to reach the summit for years |
| `bolle` | Bolle the climber | rock niche in `frost-1` (ice grip only) | deep (0.9) | easygoing, “only sat down for a moment” |
| `kiesel` | Kiesel the climber | glacier crevasse in `frost-2` | bright (1.2) | nimble, impatient, a little ashamed |
| `wicke` | Wicke the climber | ledge on the summit ridge (`frost-3`) | shaky (1.1) | timid, counts snowflakes against the fear |
| `kristella` | Ice Queen Kristella | ice hall, after the fight (`show_if = "merker besiegt.kristella"`) | calm, cool (0.8) | strict and proud, gentle and thoughtful after the fight |
| `graue-stelle` | Grey patch | summit ridge (`frost-3`) | silent, fixed | colourless ice where the Withered One drank |

The climbers vanish from their niche as soon as Elora has talked to them
(`show_if = "nicht merker kletterer.<id>"`), and afterwards sit in Flocke's hut.

## 2. Main quest “The Call of the Frostspitzen” (`frostspitzen`, replaces the “soon” step)

> The fourth spring lies in the Frostspitzen to the north. That is where the grey tracks led.

| Step | Text | Goal |
|---|---|---|
| `aufbruch` | Clear the mountain path at the top of the upper village | reach `frost-1` |
| `flocke` | Look for someone in the abandoned mountain village | talk to Flocke |
| `seil` | Fetch Flocke's rope from the frozen cellar | bring the rope to Flocke |
| `grat` | Climb up to the summit ridge with the climbing claws | reach `frost-3` |
| `quelle` | Find the ice hall of the Frost Spring | reach `frost-arena` |
| `hueter` | Calm Kristella | defeat Kristella |
| `funke` | Bring Tüftel the spring spark | spring spark to Tüftel |
| `fest` | Celebrate with Oma Pfütze at the well | talk to Oma |

Reward: 400 experience, 180 gleam drops. Afterwards the teaser **“The Song of the Stars”** begins
(`sternschlucht`, step set by hand, “soon”).

## 3. Side quests

### “Lost Climbers” (`kletterer`, from Flocke, D-M24-08)

| Step | Text | Goal |
|---|---|---|
| `finden` | Find the three missing climbers | flag `kletterer.gefunden` = 3 |
| `bericht` | Let Flocke know | talk to Flocke |

Each climber counts `kletterer.gefunden` up and sets off for the hut. Bolle in `frost-1`
sits in a niche that can only be reached with the ice grip (so only on the way back). Reward:
100 experience, 1 dewdrop point, **bobble hat**.

### “Clear Crystals for Klonk” (`klarkristalle`, from Klonk, D-M24-09)

| Step | Text | Goal |
|---|---|---|
| `sammeln` | Find eight clear crystals in the Frostspitzen | have 8 clear crystals |
| `bringen` | Bring Klonk the crystals | 8 clear crystals to Klonk |

The crystals are hidden: under thin ice, behind avalanche slopes, high up in climbing shafts.
Klonk cuts **a pendant of your choice** from them (see dialogue). Additional reward: 100 experience.

## 4. Items and Flocke's supplies

| Id | Kind | Name | Effect | Price |
|---|---|---|---|---|
| `seil` | key | Flocke's rope | “Thick, rough and with three knots that Flocke calls “luck, courage and supper”.” | – |
| `klarkristall` | collectible | Clear crystal | “Perfectly clear. Looking through it, the world seems a little calmer.” | – |
| `kraeutertee` | consumable | Herbal tea | heals 2 and **empties the cold bar** (new: effect `warm`) | 12 |
| `bommelmuetze` | hat | Bobble hat | **cold bar fills 40 % slower** | – (reward) |
| `fellstiefel` | boots | Fur boots | cold bar −20 %, armour +1 | 220 |
| `wuchtkristall` | pendant | Impact crystal | hammer +1 damage | – (Klonk) |
| `sprengkristall` | pendant | Blast crystal | explosion +20 % | – (Klonk) |
| `lichtkristall` | pendant | Light crystal | laser +1 bounce | – (Klonk) |

Flocke's supplies (shop `flocke`): herbal tea, ice crystal (25), fur boots; discount from affection 5 (10 %).
Enemies in the Frostspitzen drop ice crystals (already the case), Kristella 3 ice crystals.

## 5. Dialogues

### Flocke

**Greeting** (first visit):
> Ha! Finally someone who doesn't run away from a little snow! I'm Flocke, mountain guide. Well – mountain guide without a mountain, since the summit froze over. And without a rope. That's the real problem.

- *(curious)* “What happened to your rope?” → **Rope**
- *(friendly)* “Why is the village so empty?” → “Everyone moved down to the valley when it got colder and colder. Colder than normal, I mean. Since something grey has been stomping around up there, even the fire freezes.” → **Rope**
- *(cheeky)* “A mountain guide without a mountain? And without a rope?” → “HA! Cheeky! I like that.” (affection +1) → **Rope**

**Rope** (starts step `seil`):
> My rope is in the cellar under the old cheese dairy. I wanted to fetch it, but bats with icicles on their wings hang down there. I like bats. Just not when they fly into my face. Will you get it for me? The floor there is thin ice – don't dawdle!

- “I'll get it.” → `quest frostspitzen weiter`
- “And then?” → “Then we climb the summit! Well, you do. I'll hold the rope.” → `quest frostspitzen weiter`

**Reminder:** “The cellar under the cheese dairy. Thin ice! Don't stand still!”

**Rope returned** (step `seil`, with the rope):
> My rope! Luck, courage and supper – all three knots still there! Here, take my climbing claws for it. With them you can hold on to climbing walls – the rough, striped rocks. Jump at one, hold on, and then: jump off! Easy. Well. Almost.

→ `nimm seil 1`, `faehigkeit eisgriff`, `quest frostspitzen weiter`

- *(friendly)* “Thank you, Flocke!” → **Climbers**
- “Where to now?” → “Up! Through the chimney behind my hut. Up on the ridge it gets stormy.” → **Climbers**

**Climbers** (starts the side quest):
> Oh, and … three of my climbers haven't come back. Bolle, Kiesel and Wicke. They're good, really! But the storm … If you see them, send them home. I'll make tea.

- *(friendly)* “I'll keep my eyes open.” → `quest kletterer start`
- “Maybe later.”

**Report** (all three found):
> All three are sitting at my stove arguing about who held out the longest. Thank you, little drop. Here – my old bobble hat. With it, not even your mood will freeze.

**Calls:** “Tea is ready!” · “Cold is just warmth that happens to be somewhere else.” · “By the fire you'll warm up again!”

### Climbers
- **Bolle:** “Oh, hello! I only sat down for a moment. Three days ago. Cosy here. All right, I'll head home – Flocke is surely making tea.” → `merker kletterer.bolle = 1`, `merker kletterer.gefunden +1`
- **Kiesel:** “Don't tell Flocke! I didn't fall in, I'm … exploring the crevasse. Thoroughly. Yes, all right, I'm coming out.” → `merker kletterer.kiesel = 1`, `merker kletterer.gefunden +1`
- **Wicke:** “Four hundred and twelve … four hundred and thirteen … oh! A drop! I count snowflakes so I'm not scared. You made me lose count. Thank you. I'd better go down.” → `merker kletterer.wicke = 1`, `merker kletterer.gefunden +1`
- **In the hut** (afterwards): Bolle “Best seat by the stove.” · Kiesel “I was NOT lost.” · Wicke “It doesn't snow in here. I counted.”

### Grey patch (fixed, `frost-3`)
> The ice here is grey and dull, as if someone had drunk all the blue out of it. Next to it, a print in the snow – a large, weary foot.

### Kristella (after the fight)
> Enough. You are tougher than you look, little drop. I … was not myself. The grey one came on a clear night and drank from my spring. After that I heard nothing but storm in my head.
>
> But I saw him as he left. He was weeping. He is lonely, not wicked. He is looking for something someone took from him – and he no longer knows who.
>
> He went east, to the Sternschlucht. Take the spark. And if you find him … listen to him.

### Tüftel (step `funke`, D-M24-03)
> Blue! Ice blue! And so cool my glasses fog up. Give me your climbing claws … *clink* … *hiss* … There! Now they hold twice as long, and if you push up against the wall you even pull yourself up a bit. Climbing walls all over the Tauland – now they're yours!

→ `merker eisgriff.stark = 1`, `nimm quellfunke 1`, `quest frostspitzen weiter`

### Klonk
- **Task** (chapter 4 in progress): “Ice crystals don't melt. You know what even *they* can't do? Be as clear as clear crystals. Eight of them, somewhere in the mountains. Bring them to me and I'll cut you something pretty. Pretty and useful. I don't care about pretty.” → `quest klarkristalle start`
- **Reminder:** “Eight. Not seven. I'll count.”
- **Delivery:**
  > Eight. I counted. Twice. What'll it be?
  - “Something for the hammer.” → `gib wuchtkristall 1`
  - “Something for the grenades.” → `gib sprengkristall 1`
  - “Something for the laser.” → `gib lichtkristall 1`
  
  afterwards: “Wear it with dignity. Or at least without losing it.” → `nimm klarkristall 8`, `quest klarkristalle fertig`

### Oma Pfütze
- **Task** (after the chapter 3 celebration): “The Frost Spring, dear. When I was young it let me think so clearly that I once even understood the taxes. At the top of the upper village an ice block lies in front of the old mountain path. You know how to stomp now. And dress warmly!”
- **On the way:** “Do you have a hat? Without a hat you'll turn into an icicle.”
- **Celebration:** “Blue as a winter morning! And clear – finally clear in my head. Kristella says he is lonely? … Lonely. Like a well without a song. Oh, why does that come to mind now? Go east, dear, to the Sternschlucht. Luma the star weaver lives there. She knows every song.” → `quest frostspitzen fertig`, `quest sternschlucht start`, celebration

### Signs
- `schild-bergsteig` (upper village): “↑ Frostspitzen – mountain path. Dress warmly!”
- `schild-kaelte` (`frost-1`): “Cold makes you stiff. Warm up by a fire and under roofs.”
- `schild-eis` (`frost-1`): “Thin ice! Don't stand still. And do NOT stomp.”
- `schild-lawine` (`frost-3`): “Avalanche danger! Avoid noise and stomping.”
- `schild-kamin` (`frost-2`, behind the hut): “Climbing chimney. Climbing claws only.”

## 6. Tracks of the Withered One

Grey patches in the ice (decoration `grauspur-eis`) in `frost-1` to `frost-3`, plus the character `graue-stelle` on the
summit ridge; from the ridge, grey footprints lead east out of the arena. The Withered One does not appear.

## 7. Village after chapter 4 (outlook on M2.4.7)

`quellen_befreit = 4`, celebration; new calls from Pip (“Did you see Kristella? Is she really made of ice? All of her?”),
Lotte (“Herbal tea from the mountains – now at my shop too!”) and Tüftel (“Climbing claws, version two. Three coming soon.”).

## 8. New tech for the content

- Effect **`warm`** for consumables (empty the cold bar).
- Shop `flocke` in `shops.toml`; items in `items.toml`.
- Flag **`eisgriff.stark`**: double cling time and pulling up on the wall (tech in M2.4.7).
- Register the dialogue files in `data.rs` (`dialogs!`); characters Flocke, climbers, Kristella as graphics from the drafts.
