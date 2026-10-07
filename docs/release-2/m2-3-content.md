# R2-M2.3 – Chapter 3 content (draft, M2.3.4)

Status: **Approved and built in** (texts approved 2026-10-06) · Basis: [`m2-3-plan.md`](m2-3-plan.md), [`world-book.md`](world-book.md) §4.3 and §5, E-243, E-316 to E-325

German texts were approved; the English ones were added when building them in (quoted below as in the game). Conditions and
effects use the notation of the content files ([`../handbook/adventure-content.md`](../handbook/adventure-content.md)).

## 1. Characters

| Id | Name | Where | Voice | Nature |
|---|---|---|---|---|
| `sirup` | Sirup the caravan leader | caravan camp (`wueste-2`) | deep, easygoing (0.85) | well-travelled, loves telling long rambling stories, trades with a wink |
| `palma` | Palma, keeper of the oasis | at the oasis (`wueste-2`) | bright (1.15) | calm, worries about her oasis, speaks of water in images |
| `schlange` | Sand Serpent | arena, after the fight (`show_if = "merker besiegt.sandschlange"`) | very deep (0.5) | tired, embarrassed, warm-hearted |
| `tafel-1`, `tafel-2`, `tafel-kammer` | Ruin tablet | `wueste-3`, chamber | silent, fixed | stone tablets of the Ember Folk |
| `ruinenquelle` | Ruin spring | `wueste-3` | silent, fixed | small spring between the pillars (fill the water skin) |
| `giessstelle-1` to `-3` | Withered patch | oasis | silent, fixed | dry earth; blooms after watering (decoration `-verdorrt` → `-befreit`) |

## 2. Main quest “Tracks in the Sand” (`glutsand`, replaces the teaser)

> The third spring lies in the Glutsandwüste. A caravan tells of a grey wanderer.

| Step | Text | Goal |
|---|---|---|
| `aufbruch` | Take the sunken path off the east path into the desert | reach `wueste-1` |
| `sirup` | Find the caravan in the desert | talk to Sirup |
| `ruinen` | Follow the grey tracks to the ruins | reach `wueste-3` |
| `quelle` | Reach the Ember Spring in the sand basin | reach `wueste-arena` |
| `hueter` | Calm the Sand Serpent | defeat the Sand Serpent |
| `funke` | Bring Tüftel the spring spark | spring spark to Tüftel |
| `fest` | Celebrate with Oma Pfütze at the well | talk to Oma |

Reward: 350 experience, 160 gleam drops. Afterwards the teaser **“The Call of the Frostspitzen”** begins (step set by hand, “soon”).

## 3. Side quests

### “Water for the Oasis” (`oase`, from Palma, E-319, E-322)

| Step | Text | Goal |
|---|---|---|
| `fuellen` | Fill the water skin at the ruin spring | have 3 sips of water |
| `giessen` | Water the three withered patches of the oasis | flag `oase.gegossen` = 3 |
| `danke` | Tell Palma about the oasis | talk to Palma |

Palma hands over the empty **water skin**. At the ruin spring: full skin = **3 sips of water**. Each
withered patch takes one sip, blooms and counts `oase.gegossen` up. Reward: 80 experience,
1 dewdrop point, 3 cactus fruits.

### “The Buried Ruin” (`ruine`, from Sirup, E-323)

| Step | Text | Goal |
|---|---|---|
| `kammer` | Find a way into the buried chamber | zone `ruinenkammer` in `wueste-3` (crumbly floor, stomp only) |
| `tafel` | Read the tablet in the chamber | talk to `tafel-kammer` |
| `bericht` | Tell Sirup about the chamber | talk to Sirup |

The chamber holds a chest with the **sun veil**. Reward from Sirup: 80 experience, 1 dewdrop point,
60 gleam drops. Before the fight the chamber can only be seen through a crack; Sirup says you'd need “someone
who can stomp really hard”.

## 4. Items and Sirup's shop (E-324)

| Id | Kind | Name | Effect | Price |
|---|---|---|---|---|
| `wasserschlauch` | key | Water skin | “Made of goat leather, smells a little of camel.” | – |
| `wasser` | key | Sip of water | “Clear and cool, from the ruin spring.” | – |
| `kaktusfrucht` | consumable | Cactus fruit | heals 3 and **empties the heat bar** (new) | 12 |
| `sonnenschleier` | hat | Sun veil | **heat bar fills 40 % slower** (new) | – (reward) |
| `karawanenkette` | jewellery | Caravan chain | protection after a hit +300 ms | 180 |

Sirup's shop: cactus fruit, ember stone (material for the laser upgrade, 25), caravan chain;
discount from affection 5 (10 %), as with Lotte. Desert enemies also drop ember stone (25 % chance),
the Sand Serpent 3 ember stones instead of amber.

## 5. Dialogues

### Sirup

**Greeting** (first visit):
> Well, look at that, a drop in the embers! Welcome to Sirup's caravan – goods from every corner of the Tauland, fresh, rare and only a tiny bit sandy.

- *(curious)* “Have you seen anything strange around here?” → **Wanderer**
- *(friendly)* “What are you selling?” → shop
- *(cheeky)* “Only a tiny bit sandy? Your turban is full of sand.” → “Ho ho! The turban is an heirloom. So is the sand.” (affection +1) → **Wanderer**

**Wanderer** (advances step `sirup`):
> Strange? Three nights ago I sat by the fire, and someone came out of the dunes. All grey, like ash that has learned to walk. He knelt at Palma's oasis and drank. And drank. By morning the oasis was half empty and as pale as my grandfather after a bath.
>
> His tracks lead to the old ruins. Grey, as if someone had sucked the colour out of the sand. I'm not going there. But you have such an … adventurous look.

- “I'll follow the tracks.” → end
- “What ruins are those?” → **Ruin** (starts the side quest)

**Ruin:**
> The Ember Folk used to guard the Ember Spring there. Beneath the ruins lies a buried chamber – my father swore there's a tablet inside that nobody has ever read. The floor is old and crumbly. You'd need someone who can stomp really hard.

- *(friendly)* “I'll take a look when I can.” → `quest ruine start`
- “Maybe later.”

**Report** (step `bericht`):
> A tablet? What did it say? … He was looking for a song? Hm. Then maybe he isn't wicked at all, just lost. Like me, the first time I set off without a map. Here, for your trouble. And come back – Sirup never forgets a friend.

**Afterwards / calls:** “Fresh from the north: cactus fruit! All right, from the south.” · “Stay in the shade, little one!”

### Palma

**Greeting:**
> Quietly, please. The oasis is sleeping. It used to sing like a brook in spring. Since the grey one was here, it grows quieter every day. Three patches on the bank have already withered completely.

- *(friendly)* “Can I help?” → **Task**
- *(curious)* “Who is the grey one?” → “I only saw him from afar. He was … sad, I think. Whoever drinks like that thirsts for something water cannot quench.” → **Task**

**Task:**
> There is still a little spring in the ruins, deep between the pillars. Take my water skin. It holds enough for all three patches.

- “I'll bring water.” → `gib wasserschlauch 1`, `quest oase start`

**Reminder:** “The skin holds three sips. Each withered patch needs one.”

**Thanks** (all three watered):
> Can you hear that? It's humming again. Very softly, but it hums. Thank you, little drop. Take these fruits – they cool you when the sun presses too hard.

**Calls:** “It's cool in the shade of the palms.” · “Shhh … the oasis is humming.”

### Ruin spring (fixed)
- without the skin: “Clear water bubbles between the stones. If only you had something to carry it in …”
- with the empty skin: “You fill the water skin to the brim.” → `gib wasser 3`
- with water: “The skin is already full.”

### Withered patch 1–3 (fixed)
- without water: “Dry, cracked earth. Something used to grow here.”
- with water: “You pour a sip of water. The earth drinks greedily – and little blossoms open.” → `nimm wasser 1`, `merker befreit.giessstelle-N = 1`, `merker oase.gegossen +1`
- afterwards: “It's blooming here again.”

### Ruin tablets (fixed)
- **Tablet 1:** “We, the Ember Folk, guard the golden water. Its warmth belongs to all who are cold.”
- **Tablet 2** (picture: a grey figure bends over a spring): “Once a grey one came in the night. Where he drank, the sand turned pale. We feared him and sealed the chamber.”
- **Chamber tablet:** “Yet he did not drink out of greed. He was looking for a song he had lost – the song of the spring that came before all others.” → step `tafel`

### Sand Serpent (after the fight)
> Ssso … cool … at last. Forgive me, little drop. Something grey drank from my spring. Afterwards I was so cold, so terribly cold, that I hissed at anyone who came near.
>
> It went north. To the mountains where the snow never melts. Take the spark. And if you find him … be gentle. He sounded very cold. Colder than me.

### Tüftel (step `funke`)
> A golden spark! Warm as … no, warmer! Hold this. And this. And – stand over there. Now jump. And now DOWN! See? Stomp! It breaks crumbly floors and knocks enemies over. Try it at the practice ground, I've put something crumbly there for you.

→ `faehigkeit stampfen`, `nimm quellfunke 1`, `quest glutsand weiter`

### Klonk (after chapter 3, E-243)
> Hmph. You come back from the desert without even sand in your ears. Respect. Here. Built it from ember stone and an old lens. A laser. Don't point it at anything you like.

→ `waffe laser`, `gib munition_laser 1`, `merker klonk.laser = 1`

### Oma Pfütze
- **Task** (after the chapter 2 celebration, quest active): “The Ember Spring, dear. It warmed my hands when I was a girl. Take the sunken path south off the east path. And drink enough – the sun there isn't as friendly as I am.”
- **On the way:** “The desert won't bite you. Only the crabs will.”
- **Celebration:** “Gold! The houses glow like honey in the evening sun. And you, my brave little drop … A grey wanderer, you say? Who is looking for a song? … I don't know why, but it makes me feel quite wistful. The mountains in the north, the Frostspitzen. The fourth spring awaits there.” → `quest glutsand fertig`, `quest frostspitzen start`, celebration

### Signs
- `schild-wueste` (fork on the east path): “↓ Glutsandwüste – sunken path. Bring water!”
- `schild-treibsand`: “Careful, quicksand! If you sink in: press {taste:jump}.”
- `schild-hitze`: “Heat makes you tired. Shade under rocks and tent roofs cools you down.”
- `schild-kammer`: “The ground sounds hollow here …”

## 6. Grey tracks (E-325)

Grey footprints as decoration (`grauspur`) in `wueste-1` to `wueste-3` and at the edge of the arena: from the oasis to the
ruins and from the sand basin to the north. The wanderer does not appear.

## 7. New tech for the content

- Effect **`cool`** for consumables (empty the heat bar) and bonus **`heat_pct`** for equipment.
- Shop `sirup` in `shops.toml`.
- Register the dialogue files in `data.rs` (`dialogs!`); decoration `giessstelle-verdorrt`/`-befreit` and `grauspur` from the drafts.
