<p align="center">
  <img src="assets/elora/elora.svg" width="120" alt="Elora">
</p>

<h1 align="center">Elora</h1>

<p align="center">
  <b>Fast 2D multiplayer: hook, swing, blast – with friends online or on your LAN.<br>
  And now also on your own in an adventure.</b>
</p>

<p align="center">
  <a href="https://github.com/ehrenberg/elora/releases"><b>Download</b></a> ·
  <a href="https://elora.bastianswelt.de">Project page and live servers</a> ·
  <a href="docs/releases/v0.9.2.md">What's new in 0.9.2?</a> ·
  <a href="https://github.com/ehrenberg/elora/issues">Report a bug</a>
</p>

---

Elora is a fast 2D game in the style of the genre's great classics: nimble movement, a hook to
swing with and a handful of weapons you understand in seconds and master over weeks. Elora is
also the name of the heroine – a small, brave drop.

**Version 0.9.2 Beta** · free and open source · Linux, Windows, macOS · English and German

## What to expect

- **Hook, hammer, grenade launcher, laser** – hang on walls, swing across the map, pull opponents towards you
- **Five game modes:** Deathmatch, Team Deathmatch, Capture the Flag, Last Man Standing, Last Team Standing – each also as Instagib
- **Online and on your LAN:** server list with live status, favorites, your own server with one click
- **The adventure “The Silent Springs”** (preview): a prologue and four chapters to play on your own
- **Map editor** right in the game – build, press F5, play
- **Elora your way:** colors for eyes, body and feet, emotes for quick greetings

## Download and start

Get the latest version under **[Releases](https://github.com/ehrenberg/elora/releases)**.

| System | File | How to |
|---|---|---|
| Linux | `elora-…-linux-x86_64.AppImage` | right click → Properties → “Executable”, then double-click (or `chmod +x` in a terminal) |
| Linux | `elora-…-linux-x86_64.tar.gz` | extract, start `elora` |
| Windows | `elora-…-windows-x86_64.zip` | extract the whole ZIP (right click → “Extract all”), then start `elora.exe` in the extracted folder |
| macOS (Apple Silicon) | `elora-…-macos-aarch64.dmg` | drag Elora to “Applications”; on the first start **right click → Open** (the app is not signed) |

Elora needs a graphics card with Vulkan, DirectX 12 or Metal; without one it falls back to a
slower software renderer. There is no package for Intel Macs yet.

## First steps

After starting you land in the main menu:

| Menu | What you do there |
|---|---|
| **Play** | find and join servers online, on your LAN or from your favorites; “Quick play” takes you to the last server |
| **Adventure** | start or continue the adventure in one of three save slots |
| **Training** | practice on your own, with training dummies and all weapons |
| **Create server** | start your own server and join it right away |
| **Editor** | build your own maps |
| **Settings** | name, look, controls, graphics, sound, language |

In game, **Esc** opens the pause menu: choose a team, spectate, vote, settings.

## The adventure

The springs of the Tauland are falling silent, and the colors are fading from the village of
Tauwinkel. Elora sets out to find out why.

- **Prologue – Tauwinkel:** Oma Pfütze, Tüftel, Klonk, Lotte and Pip show you what you need
- **Chapter 1 – Blütenwiesen:** lost bees and a very grumpy Bumblebear
- **Chapter 2 – Murmelwald:** an owl full of stories, a mushroom child on its way home and the Root Warden
- **Chapter 3 – Glutsandwüste:** Sirup's caravan, quicksand, shimmering heat and the Sand Serpent
- **Chapter 4 – Frostspitzen:** mountain guide Flocke, thin ice, avalanches, biting cold and Ice Queen Kristella

Every freed spring brings a new ability (hook jerk, pull hook, stomp, ice grip) – and with it new
paths in areas you already know. There are also levels, a skill tree, equipment, shops, quests
and conversations. The game saves when you change maps and at spring stones. Weather – rain,
thunderstorms, fog, wind, sandstorms, snow – brings the areas to life and affects the adventure.
More chapters follow with the next versions.

## Playing with others

- **Join a server:** main menu → *Play*. The list shows servers online and on your LAN; you can
  also connect directly by address. You can also see which servers are running on the
  [project page](https://elora.bastianswelt.de).
- **Start a server yourself:** main menu → *Create server*, choose map and mode, start. With
  “Show on the internet” it appears in every player's list. For others to reach it, the UDP port
  (default **8303**) must be open in your router and firewall.
- **Run a server permanently:** with the program `elora-server` – all options are in the
  [guide for developers and server operators](DEVELOPMENT.md#server-console-and-master).

The connection is encrypted. Elora remembers every server and warns you if its key changes.

## Build your own maps

Main menu → *Editor*: paint terrain, choose materials such as earth, sand, snow and ice, place
decoration and backgrounds, animate clouds and trees, embed your own SVG graphics. Press **F5** to
play the map right away, Esc takes you back. Your maps then appear in *Training* and *Create
server*; when you play online, the server sends them to everyone automatically.

## Controls

Everything except Esc can be rebound under *Settings → Controls*.

| Key | Action |
|---|---|
| A / D | walk |
| Space | jump, in the air double jump |
| Right mouse button (hold) | hook |
| Left mouse button | shoot (grenade and laser: hold for continuous fire) |
| 1 / 2 / 3 or mouse wheel | hammer / grenade / laser |
| S | drop through wooden platforms; in the air: stomp (adventure, once unlocked) |
| Esc | pause menu |
| Tab (hold) | scoreboard; in the adventure: adventure menu (inventory, skills, quests, map) |
| T / Y | chat / team chat |
| Left Ctrl (hold) | emote wheel: move the mouse towards an emote, release |
| K | respawn at a spawn point (suicide) |
| F3 / F4 | yes / no in votes |
| E | talk, open, use (adventure) |
| Q | drink a healing potion (adventure) |
| Left Shift | hook jerk (adventure, once unlocked) |

After dying: the fire button respawns you quickly, otherwise you continue after three seconds. In
Last Man Standing and Last Team Standing you wait for the next round.

## Where is my data?

| | Linux | Windows | macOS |
|---|---|---|---|
| Settings | `~/.config/elora` | `%APPDATA%\Elora` | `~/Library/Application Support/Elora` |
| Save games and your own maps | `~/.local/share/elora` | `%APPDATA%\Elora` | `~/Library/Application Support/Elora` |

To back up, just copy these folders.

## If something goes wrong

| Problem | Solution |
|---|---|
| The game does not start | Look at `crash.txt` and `elora.log` in the settings folder (see above) – they tell why. On Windows the report opens in Notepad. Make sure the whole ZIP is extracted. |
| “No suitable graphics adapter” | Update your graphics driver. On Linux install the Vulkan driver (e.g. `vulkan-radeon`, `vulkan-intel` or `nvidia-utils`). If needed, force OpenGL: `WGPU_BACKEND=gl ./elora` |
| No sound | Elora also starts without audio output – then it is silent. On Linux with PipeWire, `pipewire-alsa` usually helps. |
| My server does not show up in the internet list | Open UDP port 8303 in your router and firewall; “Show on the internet” must be on. Behind DS-Lite only players with IPv6 can reach you. |
| Warning “server key changed” | The server was set up again – or someone is impersonating it. Only trust it if you know the reason. |
| The mouse cannot be captured | click into the game area; some Wayland desktops only confine the mouse instead of locking it |

Something else broken? Please report it under **[Issues](https://github.com/ehrenberg/elora/issues)**.

## Contributing

Elora is free software written in Rust. How to build it from source, test it and develop it is
described in **[DEVELOPMENT.md](DEVELOPMENT.md)**. Building maps, reporting bugs and sharing ideas
is very welcome.

## License and thanks

- Program: [GPL-3.0](LICENSE)
- Own graphics and sounds: CC-BY-SA 4.0
- Fonts (Inter, JetBrains Mono): SIL Open Font License 1.1
- Music and sounds by other artists: [`assets/SOURCES.md`](assets/SOURCES.md) and in the game under
  *Settings → About Elora*
- Libraries: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)

Inspired by [Teeworlds](https://teeworlds.com) – thanks to Magnus Auvinen and all contributors.
