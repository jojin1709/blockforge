> [!NOTE]
> **Blockforge is a hobby project.** An original Minecraft-style voxel sandbox built with Rust + Bevy. Not affiliated with Mojang/Microsoft. All textures are procedurally generated at runtime — zero external assets.

<div align="center">

# Blockforge

### An original Minecraft-style voxel sandbox, written in Rust with Bevy 0.20.

Infinite streamed terrain · Block breaking & placing · Inventory & crafting · Survival stats · Day/night cycle — **everything procedurally generated, nothing copied.**

<br/>

[![Rust](https://img.shields.io/badge/Rust-1.98-orange?logo=rust)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.20-blue?logo=bevyengine)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/jojin1709/blockforge?style=social)](https://github.com/jojin1709/blockforge)

<br/>

<sub>A clone of the Minecraft <em>gameplay loop</em> — never Minecraft's assets, name, or code.</sub>
<br/>
<sub>Developed by <strong>JOJIN JOHN</strong></sub>

</div>

---

> [!TIP]
> **Just want to play?** Skip to [Quick Start](#quick-start) — one `cargo run` and you're in.

## Table of Contents

- [Features](#features)
- [Controls](#controls)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Build & Run](#build--run)
- [How It Works](#how-it-works)
  - [World Streaming](#world-streaming)
  - [Procedural Textures](#procedural-textures)
  - [Survival Loop](#survival-loop)
  - [Lag Mitigation](#lag-mitigation)
- [Project Layout](#project-layout)
- [Roadmap](#roadmap)
- [License](#license)

---

## Features

| Feature | Status |
| --- | :---: |
| Infinite streamed chunk world (16³ chunks, 6-chunk render distance) | ✅ |
| Procedural terrain: rolling hills, caves, ore veins, trees | ✅ |
| 14-block palette (grass, dirt, stone, sand, log, leaves, planks, cobble, coal/iron/gold/diamond ore, bedrock, water) | ✅ |
| First-person controller with AABB collision, gravity, jump | ✅ |
| Break / place blocks (raycast targeting + drops into inventory) | ✅ |
| 9-slot hotbar (keys `1`–`9` + scroll wheel) | ✅ |
| 36-slot inventory (`E` to open / close) | ✅ |
| 2×2 crafting grid (logs → planks → sticks) | ✅ |
| Health & hunger bars, fall damage, apple eating, natural regen | ✅ |
| Day/night cycle with orbiting sun and sky colour lerp | ✅ |
| Sprint (`Ctrl`), sneak (`Shift`), swimming | ✅ |
| Procedural 14-tile texture atlas (nearest-neighbour, no image files) | ✅ |
| Custom block mesher with per-face UVs and face culling | ✅ |

## Controls

| Key | Action |
| --- | --- |
| `W` `A` `S` `D` | Move |
| Mouse | Look around |
| Left click | Break block |
| Right click | Place block / eat apple |
| `Space` | Jump / swim up |
| `Ctrl` | Sprint |
| `Shift` | Sneak |
| `1`–`9` | Select hotbar slot |
| Scroll wheel | Cycle hotbar |
| `E` | Open / close inventory |
| `Esc` | Close inventory (then release mouse) |

## Quick Start

### Prerequisites

- **Rust** (stable, 1.78+) — [install via rustup](https://rustup.rs)
- **Vulkan-capable GPU** (or DX12/Metal via wgpu)
- On Windows with MSVC: no extra setup needed (PDB linker workaround included in `build.rs`)

### Build & Run

```bash
git clone https://github.com/jojin1709/blockforge.git
cd blockforge
cargo run --release
```

The first launch compiles the Bevy engine and generates the texture atlas — subsequent launches start instantly.

## How It Works

### World Streaming

The world is divided into 16×16×16 chunks. Chunks around the player are generated and meshed on background threads with a **per-frame budget** (3 generations + 3 meshes per frame), keeping frame times stable even while flying across terrain. Edits (broken/placed blocks) are reapplied automatically when a chunk regenerates.

### Procedural Textures

No `.png` files exist in this repository. Every block texture — grass tops, dirt, stone grain, ore speckles, tree rings, water waves — is drawn programmatically into a single 8×8 atlas at startup using a seeded PRNG. Samples use nearest-neighbour filtering for the classic blocky look.

### Survival Loop

- **Health**: 20 points. Fall damage applies when landing from > 3 blocks. Natural regen when hunger > 17.
- **Hunger**: Drains over time (faster when sprinting). Replenished by eating apples (dropped by leaves, 12% chance).
- **Death**: Respawn at the world spawn point with full health and hunger.

### Lag Mitigation

- Chunk generation & meshing run on Bevy's async task pool with time-sliced budgets.
- UI systems (hotbar, inventory, hearts) only execute when the relevant resource actually changes — not every frame.
- Water uses a separate translucent draw pass so the opaque mesh doesn't pay transparency costs.

## Project Layout

```
src/
  main.rs       # Bevy app bootstrap + plugin registration
  blocks.rs     # Block IDs and per-face tile indices
  terrain.rs    # Procedural column generation (hills, caves, ores, trees)
  world.rs      # Chunk storage, streaming scheduler, block get/set
  mesh.rs       # Chunk mesher — opaque pass + translucent water pass
  textures.rs   # Procedural 14-tile atlas (no external images)
  player.rs     # First-person controller, collision, break/place, survival hooks
  inventory.rs  # Stacks, inventory, 2×2 crafting recipes
  ui.rs         # Hotbar, inventory screen, crosshair, tooltips
  survival.rs   # Health, hunger, fall damage, regen/starve, hearts/food bars
  daynight.rs   # Day length, sun orbit, sky/light colour lerp
replica/
  recon.md      # Game design reconnaissance (features, not code)
  features.csv  # Feature-by-feature clone matrix
```

## Roadmap

- [ ] Block breaking animation + particle effects
- [ ] Sound effects (footsteps, break, place)
- [ ] More blocks: glass, bricks, torches
- [ ] Crafting recipes: tools, chests, furnace
- [ ] Inventory drag-and-drop
- [ ] Biome variety (desert, forest, snow)
- [ ] Save / load world
- [ ] Multiplayer (LAN)

## License

All original code and assets. Released under the [MIT License](LICENSE).

Not affiliated with, endorsed by, or connected to Mojang / Microsoft / the Minecraft brand.

---

<div align="center">
<sub>Developed by <strong>JOJIN JOHN</strong> · Built with <a href="https://bevyengine.org/">Bevy</a> · <a href="https://github.com/jojin1709/blockforge">GitHub</a></sub>
</div>
