# Blockforge

An original Minecraft-style voxel sandbox game written in **Rust** with the [Bevy](https://bevyengine.org/) 0.20 engine. Infinite streamed terrain, block breaking/placing, inventory and crafting, survival stats, and a day/night cycle — all procedurally generated with zero external assets.

## Features

| Feature | Status |
|---|---|
| Infinite streamed chunk world (16³ chunks) | ✅ |
| Procedural terrain: hills, caves, trees, ore veins | ✅ |
| 14-block palette (grass, dirt, stone, sand, log, leaves, planks, cobble, coal/iron/gold/diamond ore, bedrock, water) | ✅ |
| First-person controller with AABB collision | ✅ |
| Break/place blocks (raycast + drops to inventory) | ✅ |
| 9-slot hotbar (keys 1–9 + scroll wheel) | ✅ |
| 36-slot inventory (E to open) | ✅ |
| 2×2 crafting grid (logs→planks→sticks) | ✅ |
| Health & hunger bars, fall damage, apple eating | ✅ |
| Day/night cycle with orbiting sun and sky lerp | ✅ |
| Sprint (Ctrl), sneak (Shift), swimming | ✅ |
| Procedural texture atlas (nearest-neighbour sampled) | ✅ |
| Custom block mesher with per-face UVs and face culling | ✅ |

## Controls

| Key | Action |
|---|---|
| W A S D | Move |
| Mouse | Look around |
| Left click | Break block |
| Right click | Place block / eat apple |
| Space | Jump / swim up |
| Ctrl | Sprint |
| Shift | Sneak |
| 1–9 | Select hotbar slot |
| Scroll wheel | Cycle hotbar |
| E | Open / close inventory |
| Escape | Close inventory (then release mouse) |

## Building

Requires Rust (stable) and a Vulkan-capable GPU.

```bash
cargo run --release
```

On Windows with MSVC, `build.rs` adds `/pdbpagesize:8192` to work around linker error LNK1318.

## Project layout

```
src/
  main.rs       # Bevy app + plugin registration
  blocks.rs     # Block ids and per-block face tiles
  terrain.rs    # Procedural column generation (hills, caves, ores, trees)
  world.rs      # Chunk storage, streaming scheduler, block get/set
  mesh.rs       # Greedy-ish chunk mesher (opaque + translucent water pass)
  textures.rs   # Procedural 14-tile atlas (no external image files)
  player.rs     # First-person controller, collision, break/place, survival hooks
  inventory.rs  # Stacks, inventory, 2×2 crafting recipes
  ui.rs         # Hotbar, inventory screen, crosshair, item tooltip
  survival.rs   # Health, hunger, fall damage, regen/starve, hearts/food UI
  daynight.rs   # Day length, sun orbit, sky/light colour lerp
replica/
  recon.md      # Game design reconnaissance (features, not code)
  features.csv  # Feature-by-feature clone matrix
```

## Design notes

- **No external assets** — every texture is generated at startup into a single atlas; terrain uses the `noise` crate.
- **Streaming** — chunks load/unload around the player with a per-frame budget (3 gens + 3 meshes/frame) to keep frame times stable.
- **Two-pass rendering** — opaque chunks first, then translucent water meshes (alpha blend, no back-face culling).
- **Lag-conscious** — UI systems only run when inventory/crafting state actually changes; meshing and generation are time-budgeted.

## License

All original code and assets. Not affiliated with or endorsed by Mojang/Microsoft.
