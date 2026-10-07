# M1 – Physics Sandbox: Implementation Plan

Status: **completed** (E-049) · accepted (E-043–E-046) · Basis: [`06-roadmap.md`](06-roadmap.md) M1, E-013

## Goal

Elora runs, jumps, double-jumps and hooks on the test map. All tuning values can be adjusted live. Acceptance is passed when you confirm that the movement feels right.

## Work Steps

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M1.1 ✅ | Simulation core | `elora-sim` | `Vec2` with quantization (E-021), `Tuning` (E-023), tile collision (`MoveBox`, raycast for the hook), movement, jump, double jump, velocity ramp, hook state machine | Unit tests: recompute jump heights from `../../handbook/tuning.md`, hook range, no tunneling through walls |
| M1.2 ✅ | Map loader | `elora-map` | Parser for `.emap.toml` (E-024), validation with line and column, test map `maps/sandbox.emap.toml` | Tests: valid and invalid maps |
| M1.3 ✅ | Window & renderer | `elora-render` | winit window, wgpu, shapes via lyon (circle, rectangle, line; E-033), camera with view area (D-02) | Visual check |
| M1.4 ✅ | Game loop & input | `elora-client` | Fixed tick at 50 TPS with accumulator, interpolation between ticks, key bindings (D-05), mouse aiming, camera (D-01) | The sandbox is playable |
| M1.5 ✅ | Debug tools | `elora-client` | egui panel: all tuning values as sliders, display of velocity, ground contact and hook state, map hot reload, key for reset/respawn, saving the tuning (D-03) | Manual |
| M1.6 ✅ | Determinism | `elora-sim` | Input recording → golden file with the final state, test in `cargo xtask check` | Test green |
| M1.7 ✅ | Acceptance | – | You play the sandbox, possibly with re-tuning. The final values go into `../../handbook/tuning.md`. | Your acceptance |

After each step: `cargo xtask check` green, then a commit.

## Technical Specifications (Proposal)

- **Math:** our own small `Vec2` in `elora-sim` instead of glam. This gives us full control over every float operation and the rounding, which matters for determinism (E-021). `elora-sim` stays free of external dependencies.
- **Porting:** Movement (`CCharacterCore::Tick`/`Move`) and `MoveBox` follow the Teeworlds code as reference (E-007), with our tuning values.
- **Dependencies, stable versions only:** wgpu, winit (stable 0.30 series instead of the 0.31 beta), egui, egui-wgpu, egui-winit, lyon, toml and serde, notify for hot reload, tracing, anyhow/thiserror, pollster. The exact versions follow from what is mutually compatible.
- **Placeholder look:** Elora is a circle with a facing direction, tiles are colored rectangles by type, the hook is a line, plus a crosshair.

## Decisions for M1

| # | Question | Original behavior (0.7) |
|---|---|---|
| D-01 → E-044 | Camera | Static (default): camera exactly on the character, crosshair max. 400 units away. Optionally dynamic with dead zone 300, follow factor 60 %, max. mouse distance 1000 |
| D-02 → E-045 | View area | View area 1150 × 1000 = 1.15 million units², at most 1500 × 1050 units. The aspect ratio determines width and height. |
| D-03 → E-046 | Saving tuning | – (a server setting in the original) |
| D-05 → E-043 | Key bindings | A/D run, Space jump, LMB shoot, RMB hook |

## Implementation Notes

- **Starting the sandbox:** `cargo run --bin elora` (default map `maps/sandbox.emap.toml`) or `cargo run --bin elora -- pfad/zur/karte.emap.toml`.
- **Controls:** A/D, Space, RMB (hook), R respawn, F1 panel, F5 recording, Esc release mouse / quit.
- **Tuning:** sliders in the panel; *Save* writes `tuning.toml` in the working directory (E-046).
- **Recordings (M1.6):** F5 puts Elora on the spawn and records all inputs; pressing F5 again (or R, a map change, a tuning change) stops and saves to `crates/elora-sim/tests/recordings/rec-<zeit>.erec.toml`. The file contains grid, spawn, tuning and inputs. Generate the golden file: `ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features`. After that, `cargo xtask check` verifies on every change that the simulation stays bit-identical.
- **Placeholder:** Elora is drawn at hitbox size (28); the later display size is part of M5.
