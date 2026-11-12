# R2-M2.6 – Finale content (draft, M2.6.4)

Status: **approved (E-383)** · Basis: [`m2-6-plan.md`](m2-6-plan.md), [`world-book.md`](world-book.md) §2, §4.6 and §5, E-380 to E-382

As for chapters 4 and 5 (E-346, E-377) the **German texts are up for approval**; the English
ones are added when building them in. Conditions and effects use the notation of the content
files ([`../handbook/adventure-content.md`](../handbook/adventure-content.md)). Open choices are
marked **D-M26C-xx** with a proposal.

## 1. The well song

The song Oma's mother sang at the well. Oma knows the first line (end of chapter 5); each of the
five guardians remembers one more line (D-M26-01). In the finale Elora plays it on five note
stones, one note per guardian line (D-M26-05).

| Line | Who remembers it | Text (de) |
|---|---|---|
| 1 | Oma Pfütze | Tief unten, wo das Wasser schläft, |
| 2 | the Bumblebear (Blütenwiesen) | da hütet einer still das Licht. |
| 3 | the Root Warden (Murmelwald) | Er gab den Quellen ihren Tau, |
| 4 | the Sand Serpent (Glutsandwüste) | doch hören konnten sie ihn nicht. |
| 5 | Kristella (Frostspitzen) | Komm, sing mit ihm, sing ihn wach – |
| 6 | Luma (Sternschlucht) | dann fließen alle Farben nach. |

**D-M26C-01 Song:** six lines as above – **approved** (E-383).

**D-M26C-02 Tune:** five notes, one per guardian line, in the spring colours of the note stones
(green, brown, gold, ice blue, violet): **`c d e g c2`**, rising like waking up (E-383).

## 2. Characters

| Id | Name | Where | Voice | Nature |
|---|---|---|---|---|
| `withered` | Der Dürre | heart of the Origin Cave (`origin-heart`), curled up | low, slow (0.7) | tired, sad, speaks little; wakes up a little with each colour |
| `born` | Born | at the open well in Tauwinkel after the finale | deep, warm (0.75) | old, calm, gentle humour; thankful; offers the spring trials (E-381) |
| `sign-shaft` | Signpost | top of `well-shaft` | – | hint for the way down |

The five guardians and Oma keep their characters; they get new nodes.

## 3. Main quest “The Old Well” (`finale`, replaces the “soon” step)

> Almost everything in the Tauland is colourful again – only the old well in Tauwinkel stays
> grey. Luma says the grey one hummed a well song as he left.

| Step | Text (de) | Goal |
|---|---|---|
| `song` | Die Wächter nach dem alten Brunnenlied fragen | flag `well_song.lines` = 5 |
| `well` | Oma das Brunnenlied bringen | talk to Oma |
| `shaft` | In den Brunnen hinabsteigen | reach `well-shaft` |
| `veins` | Die fünf Farbtropfen in der Urquellen-Höhle finden | flag `finale.drops` = 5 |
| `heart` | Ins Herz der Höhle gehen | reach `origin-heart` |
| `colours` | Dem Dürren die fünf Farben zurückbringen | flag `finale.colours` = 5 |
| `sing` | Dem Dürren das Brunnenlied vorspielen | flag `finale.done` |
| `home` | Mit allen am Brunnen feiern | talk to Oma |

Reward: 800 experience, 300 gleam drops. Afterwards free play; Born offers the trials.

## 4. Items

| Id | Kind | Name (de) | Effect / description |
|---|---|---|---|
| `drop_blossom` … `drop_star` | (carried, not in the bag) | Grüner / Brauner / Goldener / Eisblauer / Violetter Farbtropfen | „Ein Tropfen Farbe, der leise summt. Er will zurück.“ |
| `dew_crown` | hat | Tau-Krone | +1 health, glows in all spring colours – „Von Born. Sie ist kühl wie ein Morgen am Brunnen.“ |

**D-M26C-03 Keepsakes of the guardians:** **none** (E-383) – only Born's dew crown. The proposal
below is not built:

| From | Id | Name (de) | Description (de) |
|---|---|---|---|
| Bumblebear | `keepsake_honey` | Honigtropfen | „Ein Tropfen vom ersten Honig nach dem Fest.“ |
| Root Warden | `keepsake_leaf` | Moosblatt | „Es raschelt, wenn jemand eine Geschichte erzählt.“ |
| Sand Serpent | `keepsake_scale` | Goldschuppe | „Warm, auch im Winter.“ |
| Kristella | `keepsake_snowflake` | Ewige Schneeflocke | „Sie schmilzt nicht. Kristella hat es ihr verboten.“ |
| Luma | `keepsake_thread` | Sternenfaden | „Er leuchtet ein bisschen, wenn du träumst.“ |

## 5. Dialogues

### Oma Pfütze

**Start of the finale** (after the chapter 5 celebration, step `song`):
> Ein Brunnenlied … Weißt du, Liebes, die Wächter sind alt. Älter als ich. Älter als Tauwinkel. Wenn jemand dieses Lied noch kennt, dann sie. Frag sie – jeden einzelnen. Und bring mir, was sie noch wissen.

- *(freundlich)* „Ich frage sie alle.“ → `quest finale start` (already running), flag `well_song.oma = 1`
- *(neugierig)* „Warum ist der Brunnen überhaupt zu?“ → „Das war schon immer so. Die Gründer haben einen Deckel draufgemacht. Warum … das hat mir nie jemand erzählt. Komisch, nicht?“

**On the way** (step `song`, fewer than five lines):
> Wie viele Zeilen hast du schon? *(zählt an den Fingern)* … Es fehlen noch welche. Die Wächter wohnen weit, aber du hast ja jetzt Flügel. Fast.

**The song** (step `well`, all five lines):
> Alle Zeilen … Komm, stell dich neben mich.
>
> *(singt)* „Tief unten, wo das Wasser schläft, da hütet einer still das Licht. Er gab den Quellen ihren Tau, doch hören konnten sie ihn nicht. Komm, sing mit ihm, sing ihn wach – dann fließen alle Farben nach.“
>
> … Hörst du das? Der Brunnen. Er singt zurück.

→ flag `well.open = 1` (the lid lifts, decor `well` → `well-open`), next step

- *(neugierig)* „Da unten ist er, oder?“ → „Ja. Ich glaube, er war schon immer da. Und wir haben einen Deckel auf ihn gelegt.“
- *(freundlich)* „Ich hole ihn.“ → „Nicht holen, Liebes. Besuchen. Und nimm das Lied mit.“

**Before the descent** (step `shaft`): „Pass auf dich auf, Kind. Und sag ihm … sag ihm, dass wir ihn vergessen haben. Nicht, weil er uns egal war.“

**Celebration** (step `home`, after the ending scene):
> Jetzt weiß ich es wieder. Alles. Meine Mutter hat es jeden Morgen hier gesungen – damit er nicht allein ist. Irgendwann hat es niemand mehr gesungen. Und dann hat jemand den Deckel draufgemacht, weil man ja nicht in einen Brunnen fallen soll …
>
> Von jetzt an singen wir es wieder. Jeden Morgen.

→ `quest finale done`; the dew crown comes from Born (below)

### The guardians (one line each, after their chapter, while step `song` runs)

Each node: `flag well_song.<guardian> = 1`, `flag well_song.lines +1`; afterwards the guardian
repeats the line as a call.

- **Bumblebear** *(brummt nachdenklich)*: „Ein Lied am Brunnen? Brumm … Ich war klein, ganz klein. Da hat eine Stimme gesungen: ‚… da hütet einer still das Licht.‘ Brumm. Mehr weiß ich nicht.“
- **Root Warden**: „Ich erinnere mich. Ich erinnere mich an alles, das ist meine Aufgabe. ‚Er gab den Quellen ihren Tau.‘ Wir alle haben unseren Tau von ihm. Wir haben es vergessen. Ich nicht – aber ich habe es niemandem gesagt. Das tut mir leid.“
- **Sand Serpent** *(rollt sich zusammen)*: „Sssie haben ihn nicht gehört. ‚Doch hören konnten sssie ihn nicht.‘ Ich habe ihn auch nicht gehört. Ich lag in der Sonne und dachte, die Wärme kommt von allein.“
- **Kristella**: „‚Komm, sing mit ihm, sing ihn wach.‘ – Das ist keine Bitte an ihn, kleiner Tropfen. Das ist eine Bitte an dich.“
- **Luma** (at the ruins): *(summt)* „‚… dann fließen alle Farben nach.‘ Das ist das Ende. Jetzt hast du alles. Ich webe dir die Melodie in die Erinnerung, damit du sie nicht verlierst.“

**D-M26C-04 Order:** **fixed** (E-383): from chapter 1 to 5 – a guardian only remembers their line
once the line before is known (`well_song.lines` = their number − 1); before that they say they
do not know where the song begins and send Elora to the one before.

### Pip (when the well opens)
- „Ich komm mit! Ich kann auch … also … ich kann sehr gut zugucken!“ – Oma: „Nein.“ – Pip: „… Okay.“

### The Withered One (heart of the cave)

**Arrival** (before the rounds): „… Niemand kam. Niemand sang. Ich habe gewartet, bis die Farben weg waren. Dann habe ich sie mir geholt. Aber sie sind nicht warm geworden.“

- *(freundlich)* „Ich bringe sie dir zurück. Alle.“ → rounds start (`flag finale.ready = 1`)
- *(neugierig)* „Wer bist du?“ → „Ich weiß es nicht mehr. Ich hatte einen Namen. Den hat man mit dem Deckel zugedeckt.“ → rounds start

**After each colour** (short line, shown above him):
1. „… Grün. Wie Gras am Morgen.“
2. „Braun. Erde. Ich weiß wieder, wie Erde riecht.“
3. „Gold … warm. Warum ist mir warm?“
4. „Blau. Klar. Ich kann wieder denken.“
5. „Violett … Ich habe geträumt. Von einem Lied.“

**Failure** (grey waves fill the heart, Elora wakes at the spring stone): „… Du bist noch da?“ (when she comes back)

**After the song** (stage 2 done):
> *(öffnet die Augen ganz)* Das … das ist mein Lied. Jemand hat es gesungen. Für mich.
>
> Ich heiße Born. Ich bin die Quelle unter den Quellen. Und ich war so lange allein, dass ich vergessen habe, wer ich bin.
>
> Danke, kleiner Tropfen.

→ ending scene

### Born (at the open well, after the finale)

**First talk** (after the celebration):
> Weißt du, was das Schönste ist? Jeden Morgen singt jemand. Manchmal falsch. Das ist mir egal.
>
> Hier – eine Krone aus Tau. Damit du immer ein bisschen nach Zuhause leuchtest.

→ `give dew_crown 1`

**Trials** (E-381, every later talk):
> Die Quellen sind wach – und ein bisschen übermütig. Sie haben ihre Wege schwerer gemacht. Willst du es versuchen? Ich zähle mit.

- „Zeig mir die Prüfungen.“ → trial menu
- „Erzähl mir von früher.“ → „Früher? Da war hier nur Wasser und Stein. Dann kamen die ersten Tropfen und haben gesungen. Dann haben sie aufgehört. Jetzt singen sie wieder. Mehr Geschichte braucht es nicht.“
- „Bis später, Born.“

**After a trial:** gold: „Gold! Die Quelle staunt.“ · silver: „Silber. Schön schnell.“ · bronze: „Bronze. Geschafft ist geschafft.“ · no medal: „Geschafft! Beim nächsten Mal glänzt es mehr.“ · ended early: „Nicht schlimm. Die Wege laufen nicht weg.“

**Calls:** „Jeden Morgen ein Lied.“ · „Pass auf, der Brunnen spritzt manchmal.“

### Village after the finale (calls)
- **Pip:** „Ein Hüter im Brunnen! Ich hab's IMMER gewusst!“
- **Lotte:** „Brunnenwasser in Flaschen? … Nein. Das wäre irgendwie unhöflich.“
- **Klonk:** „Den Deckel schmelz ich ein. Daraus wird … hmpf. Eine Bank. Am Brunnen.“
- **Tüftel:** „Wenn man das Brunnenwasser … nein. Manche Dinge lässt man einfach, wie sie sind.“
- **Glimmer:** „Hier ist es jetzt noch heller. Ich bleib wirklich für immer.“

### Signs
- `sign-shaft` (top of `well-shaft`): „Tief. Sehr tief. Halt dich an den Wurzeln fest.“

## 6. Victory screen (D-M26-08)

- Text: „Born singt wieder – und mit ihm das ganze Tauland.“
- Title of honour: **„Quellenhüterin“**

**Too early** (fixed order, E-383): every guardian except the Bumblebear says, with the name of the
guardian before: „Ein Lied? Ich weiß nur ein Stück aus der Mitte – ohne den Anfang finde ich es
nicht. Frag zuerst {Hummel / Wurzelwächter / Sandschlange / Kristella}.“ (approved 2026-11-12)

## 7. Approved

All texts and the choices D-M26C-01 to D-M26C-04 (E-383, 2026-11-12). The “too early” line of the
guardians was approved separately (above).
