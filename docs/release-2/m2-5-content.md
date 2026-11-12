# R2-M2.5 – Chapter 5 content (draft, M2.5.6)

Status: **draft – waiting for approval** · Basis: [`m2-5-plan.md`](m2-5-plan.md), [`world-book.md`](world-book.md) §4.5 and §5, E-362 to E-369

As for chapter 4 (E-346), the **German texts are up for approval**; the English ones are added when
building them in. Conditions and effects use the notation of the content files
([`../handbook/adventure-content.md`](../handbook/adventure-content.md)). Open choices are marked
**D-M25-xx** with a proposal.

## 1. Characters

| Id | Name | Where | Voice | Nature |
|---|---|---|---|---|
| `luma` | Luma the star weaver | her hollow in `gorge-2`, later at the ruins (`gorge-spring`) | soft, a little sung (1.15) | gentle, tired, speaks in images; weaves threads of light; last guardian of the vanished Star Spring (draft B, E-369) |
| `glimmer` | Glimmer the glow-worm | at the gorge entrance (`gorge-1`) | bright, fast (1.3) | timid, lights up when frightened; lost his way in the dark |
| `grey_trail_stone` | Grey stone | `gorge-1` and `gorge-3` | silent, fixed | grey patches where the Withered One sat down |
| `sign-gorge` | Signpost | east end of `meadow-3` | – | “Sternschlucht” – closed before chapter 4 (E-354) |

Luma appears in her hollow until the climax starts (`show_if = "not flag luma.at_ruins"`) and
afterwards stands at the ruins.

## 2. Main quest “The Song of the Stars” (`sternschlucht`, replaces the “soon” step)

> Kristella saw the grey one head east, to the Sternschlucht. Luma the star weaver lives there.
> She knows every song.

| Step | Text (de) | Goal |
|---|---|---|
| `departure` | Den Felsspalt am Ostende der Blütenwiesen finden | reach `gorge-1` |
| `luma` | Im Dunkeln nach der Sternenweberin suchen | talk to Luma |
| `threads` | Drei Lichtfäden für Luma sammeln | have 3 light threads |
| `cape` | Luma die Lichtfäden bringen | 3 light threads to Luma |
| `heights` | Mit dem Sternenumhang hinauf in die dunklen Höhen | reach `gorge-3` |
| `ruins` | Die Ruinen der Sternquelle finden | reach `gorge-spring` |
| `protect` | Luma beschützen, bis ihr Lied zu Ende ist | flag `luma_protected` = 1 |
| `spark` | Tüftel den Sternenfunken bringen | spring spark to Tüftel |
| `party` | Mit Oma Pfütze am Brunnen feiern | talk to Oma |

Reward: 500 experience, 200 gleam drops. Afterwards the teaser **“The Old Well”** begins
(`finale`, step set by hand, “soon”).

**D-M25-01 Light threads:** the three threads hang at the end of star bridges in `gorge-1` and
`gorge-2` – the note stones are taught on the way there (proposal). Alternative: Luma weaves the
cape right away (shorter, but the note stones have no purpose before the song).

## 3. Side quests

### “Star Fragments” (`star_fragments`, from Luma)

| Step | Text (de) | Goal |
|---|---|---|
| `collect` | Acht Sternsplitter in der Sternschlucht finden | have 8 star fragments |
| `bring` | Luma die Sternsplitter bringen | 8 star fragments to Luma |

The fragments lie in places only reachable by gliding: across the gorge, on drifting islands, at
the top of updrafts. Reward: 150 experience, 1 dewdrop point, **star shawl** (cape: gliding falls
30 % slower, +1 health).

### “Luma's Song” (`luma_song`, from Luma, E-368)

| Step | Text (de) | Goal |
|---|---|---|
| `find` | Die drei Strophen von Lumas Lied auf den Notensteinen spielen | flag `luma_song.verses` = 3 |
| `sing` | Luma ihr Lied vorspielen | talk to Luma |

Each map `gorge-1` to `gorge-3` has a **note tablet** (sign) that shows a short tune as coloured
notes; playing it on the note stones of that map counts `luma_song.verses` up once (new: tune check
in the session with `Session::played`). Then Luma's song is complete: flag **`luma_song`** = 1 –
in the climax she sings 1.5 × faster (M2.5.5). Reward: 120 experience, **song pendant** (pendant:
star lanterns burn 50 % longer).

**D-M25-02 Tunes:** three notes per verse, for example verse 1 `e g c2`, verse 2 `c e g`, verse 3
`g e c` (proposal: short so it stays a puzzle, not a test of memory).

## 4. Items

| Id | Kind | Name (de) | Effect | Price |
|---|---|---|---|---|
| `light_thread` | collectible | Lichtfaden | „Warm und leicht wie ein Sonnenstrahl, den jemand aufgewickelt hat.“ | – |
| `star_fragment` | collectible | Sternsplitter | „Ein Stück Himmel, kaum größer als ein Tautropfen. Es summt leise.“ | – |
| `star_cape` | (ability) | Sternenumhang | ability **glide** (E-363) – not an item, Luma weaves it right onto Elora | – |
| `star_shawl` | cape | Sternenschal | gliding falls 30 % slower, +1 health | – (reward) |
| `song_pendant` | pendant | Liedanhänger | star lanterns burn 50 % longer | – (reward) |
| `night_tea` | consumable | Nachttee | heals 2 and Elora's light shines twice as far for 30 s | 18 (Lotte) |

**D-M25-03 Night tea:** a small helper for players who find the gorge too dark (proposal:
Lotte sells it after Elora has been in the gorge once; needs the new effect `glow`).

## 5. Dialogues

### Glimmer (gorge entrance, `gorge-1`)
> *(blinkt hell auf)* Huch! Ach, du bist es nur. Also – jemand. Ein Tropfen. Entschuldige, wenn ich mich erschrecke, leuchte ich. Und ich erschrecke mich hier ständig.

- *(neugierig)* „Was ist hier unten so gruselig?“ → „Es ist dunkel geworden. Richtig dunkel. Früher hat die Sternquelle die ganze Schlucht beleuchtet, aber jetzt … Und manchmal sitzt da hinten etwas Graues und seufzt.“ → `flag withered_one.gorge = 1`
- *(freundlich)* „Ich bring dich hier raus.“ → „Wirklich? Dann geh ich einfach hinter dir. Ganz dicht. Nicht umdrehen.“
- *(frech)* „Du bist doch die Lampe. Geh du vor!“ → „Ich? Vorgehen? *(blinkt dreimal)* … Nein.“ (affection +1)

**Calls:** „Nicht umdrehen!“ · „Hast du das auch gehört?“ · „Die Pilze leuchten, ich leuchte – wir sind ein Team.“

### Luma (hollow in `gorge-2`)

**Greeting** (first visit, starts step `threads`):
> Ein Tropfen … mit Farbe. Wie lange habe ich so etwas nicht gesehen. Ich bin Luma. Früher habe ich Lieder aus Sternenlicht gewebt. Jetzt reichen meine Fäden kaum noch für mich selbst.
>
> Der Graue war hier. Er hat aus meiner Quelle getrunken, bis nur noch ein Schimmer übrig war. Und er hat dabei geweint.

- *(freundlich)* „Kann ich dir helfen?“ → **Threads**
- *(neugierig)* „Der Graue weint?“ → „Er sucht etwas. Ich glaube, er weiß selbst nicht mehr, was. Wer so lange allein ist, vergisst, wonach er sucht.“ → **Threads**
- *(frech)* „Du siehst aus, als bräuchtest du selber Hilfe.“ → „*(lacht leise)* Das stimmt. Dann helfen wir uns gegenseitig.“ (affection +1) → **Threads**

**Threads:**
> In der Schlucht hängen noch drei Lichtfäden – am Ende der Sternbrücken. Spiel die Notensteine, dann tragen dich die Brücken hin. Bring mir die Fäden, und ich webe dir etwas, mit dem du fliegen kannst. Fast.

- „Ich hol sie.“ → `quest sternschlucht next`

**Reminder:** „Die Notensteine singen, die Brücken hören zu. Drei Fäden, kleiner Tropfen.“

**Cape** (step `cape`, with three threads):
> Drei Fäden. Das reicht für einen Umhang. Halt still … *(summt)* … So. Der Sternenumhang. Wenn du in der Luft bist, halt den Sprung gedrückt – dann trägt er dich. Weit. Bis zu den dunklen Höhen.

→ `take light_thread 3`, `ability glide`, `quest sternschlucht next`

- *(freundlich)* „Danke, Luma!“ → **Side quests**
- „Und du?“ → „Ich gehe zu den Ruinen der Quelle. Mit dem letzten Schimmer kann ich vielleicht ein letztes Lied singen. Komm nach, wenn du kannst.“ → `flag luma.at_ruins = 1` → **Side quests**

**Side quests:**
> Zwei Dinge noch. Der Graue hat Sternsplitter verstreut, als er trank – acht Stück, an Stellen, die man nur fliegend erreicht. Und mein Lied … ich habe die Strophen vergessen. Sie stehen noch auf den Notentafeln der Schlucht. Spielst du sie für mich?

- „Ich halte die Augen offen.“ → `quest star_fragments start`, `quest luma_song start`
- „Vielleicht später.“

**Fragments returned:** „Acht Stück Himmel. Daraus webe ich dir einen Schal – damit fällst du noch sanfter.“ → `take star_fragment 8`, `give star_shawl 1`, `quest star_fragments done`

**Song complete:**
> *(hört zu, die Augen geschlossen)* Ja … so ging es. Jetzt weiß ich es wieder. Mit diesem Lied kann ich viel länger singen. Hier – damit deine Laternen länger brennen.

→ `give song_pendant 1`, `quest luma_song done`, `flag luma_song = 1`

**At the ruins** (before the climax starts):
> Hier war meine Quelle. Siehst du den letzten Funken? Ich werde singen, um ihn zu halten. Der Graue wird kommen – er wird es spüren. Zünde die Sternenlaternen an. Ihr Licht hält seine Schatten fern.

**After the climax** (Luma at the ruins):
> Er hat dich angesehen. Nicht böse – traurig. Er nimmt die Farben nicht für sich. Er sucht etwas, das er verloren hat.
>
> Und weißt du, welches Lied er gesummt hat, als er ging? Ein Brunnenlied. Ein sehr altes. Er ist nach Westen gegangen. Nach Tauwinkel. Nimm den Funken, kleiner Tropfen – und geh nach Hause.

→ `give spring_spark 1`, `quest sternschlucht next`

### Tüftel (step `spark`)
> Violett! Das ist … das ist ja Traumfarbe! Gib mal deinen Umhang … nein, warte … ja! *zisch* … *sirr* … So! Jetzt steigt er in Aufwinden doppelt so hoch, und du kannst im Gleiten einmal kurz nach oben flattern. Probier's an den alten Stellen aus – in den Wiesen, im Wald, überall gibt es jetzt Wege nach oben!

→ `flag glide.strong = 1`, `take spring_spark 1`, `quest sternschlucht next`

**D-M25-04 Stronger gliding:** “once a short flap upwards while gliding” plus double lift in
updrafts (proposal). The tech is built (M2.5.8) behind the flag `glide.strong` with the values
A-52 (flap 8 units/tick, about two tiles, once per flight) and A-53 (updraft × 1.5 while gliding,
about twice as high out of an updraft) – to try out in the sandbox: F1 → tuning “Abilities & enemies” →
“Strengthened star cape” (with gliding switched on under “Abilities (adventure)”).

### Oma Pfütze
- **Task** (start of the chapter, after the chapter 4 celebration, already in the game): „Geh nach Osten, Liebes, zur Sternschlucht …“
- **On the way:** „In der Schlucht ist es dunkel, Liebes. Bleib bei den Pilzen. Und hab keine Angst vor dem Dunkeln – es ist nur Licht, das gerade schläft.“
- **Celebration:**
  > Alles bunt! Fast alles. Nur der alte Brunnen … der bleibt grau, egal, wie viele Quellen wir befreien. Ein Brunnenlied, hat Luma gesagt? Warte … *(summt)* … „Tief unten, wo das Wasser schläft, …“ – wie ging es weiter? Ich weiß es nicht mehr. Aber ich weiß, dass meine Mutter es gesungen hat. Genau hier.

  → `quest sternschlucht done`, `quest finale start`, celebration

### Village after chapter 5 (calls)
- **Pip:** „Du kannst FLIEGEN? Fast fliegen? Nimmst du mich mal mit?“
- **Lotte:** „Nachttee! Frisch aus der Schlucht. Na ja, frisch gemischt.“
- **Klonk:** „Der Brunnen ist grau. Den hab ich nicht geschmiedet. Hmpf. Den hat keiner geschmiedet.“
- **Tüftel:** „Gleiten, Version zwei. Version drei: Rakete. Vielleicht.“
- **Glimmer** (moved to the village, by the well): „Hier ist es hell. Ich bleib hier. Für immer. Oder bis morgen.“

### Signs and note tablets
- `sign-gorge` (east end of `meadow-3`): before chapter 4 „Sternschlucht – gesperrt. Steinschlag.“, afterwards „→ Sternschlucht. Licht mitnehmen!“
- `sign-dark` (`gorge-1`): „Dunkel? Leuchtpilze und Sternenlaternen machen Licht. Schattenmotten mögen kein Licht.“
- `sign-glide` (`gorge-2`, after the cape): „Sprung in der Luft halten: gleiten. Aufwinde tragen dich nach oben.“
- `tablet-song-1` to `tablet-song-3`: „Lumas Lied, Strophe 1: 🔵 🟢 🟡“ (notes as coloured dots in the colour of the note stones)

## 6. Tracks of the Withered One

Grey patches (decoration `grey_trail_ice` in a stone variant) in `gorge-1` and `gorge-3`, the
character `grey_trail_stone`; in the climax he appears for the first time (M2.5.5).

## 7. Village after chapter 5 (outlook on M2.5.8)

`springs_freed = 5`, celebration; everything colourful except the old well; Glimmer at the well.

## 8. New tech for the content

- ✅ Session: **tune check** – playing the tune of a note tablet on the note stones of its map counts a
  flag up once (for “Luma's Song”): `[[tune]]` in `quests.toml` with `map`, `notes`, optional `if`
  and `do` (see [`../handbook/adventure-content.md`](../handbook/adventure-content.md)); the three
  verses are entered once D-M25-02 is decided.
- ✅ Effect **`glow`** for consumables (Elora's light reach, D-M25-03): twice as far for 30 s.
- ✅ Pendant bonus **lantern time** (`lantern_pct`) for the song pendant.
- Register the dialogue files; characters Luma, Glimmer, grey stone as graphics from the drafts
  (Glimmer is new: small glow-worm, drawn in the style of the chapter 5 drafts).
