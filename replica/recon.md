# Recon map: Minecraft (PC/Java Edition) → "Blockforge"

Scope: the core survival loop — world, mining/building, inventory, crafting, day/night, simple mobs.
For: the user's own game (Rust + Bevy, own name/brand/assets).
Date: 2026-10-13

## Sources

| # | source | URL | notes |
| --- | --- | --- | --- |
| 1 | Official wiki (blocks, crafting, mobs) | https://minecraft.wiki | most complete public feature list |
| 2 | Marketing site | https://www.minecraft.net | positioning, core loop in their words |
| 3 | Changelog / patch notes | https://www.minecraft.net/en-us/article/minecraft-java-edition-… | what the team ships |
| 4 | App/game store listing | Steam / Microsoft Store page | screenshots of key screens |
| 5 | Public walkthrough videos | YouTube "Minecraft survival gameplay" | real flows click by click |
| 6 | Bevy engine docs | https://bevy.org/learn/quick-start/getting-started/ | our engine (0.20) |
| 7 | block-mesh crate | https://crates.io/crates/block-mesh | greedy meshing, Bevy-independent |
| 8 | bevy_voxel_world (pattern ref) | https://github.com/splashdust/bevy_voxel_world | chunk pipeline pattern, AO, LOD ideas | |
| 9 | noise crate | https://crates.io/crates/noise | simplex/perlin terrain gen |

## Core loop

Explore a blocky infinite world, mine resources, craft better tools, build and survive the night.

## Screens

| ID | screen | route / how to reach | purpose | key components | states seen |
| --- | --- | --- | --- | --- | --- |
| S01 | Main menu | launch | title, play/options/quit | buttons, logo, splash text | default, hover |
| S02 | World select | Play → | list saves, create/delete | list, "Create New", seed input | empty (no worlds), filled |
| S03 | World creation | World select → Create | name, seed, gamemode, cheats | form fields, toggles | defaults, custom |
| S04 | Loading terrain | after create/load | chunk gen progress | progress bar, tips | loading, stuck/error |
| S05 | In-game world | S04 done | the game itself | HUD, crosshair, hotbar, hearts, hunger | day, night, underwater, in cave |
| S06 | Inventory | E key | items, 2×2 craft, armor | grid, slots, drag-drop | empty, filled |
| S07 | Crafting table | right-click table, 3×3 | recipes | 3×3 grid, result slot | no match, match |
| S08 | Pause menu | Esc | resume, settings, save+quit | buttons | in-game |
| S09 | Settings | Pause → Options | video/audio/controls, render distance | sliders, toggles | defaults, changed |
| S10 | Death screen | health = 0 | respawn or title | buttons | death |
| S11 | Chat/commands | T or / | talk, cheat commands | text input | open, closed |

## Flows

```
F01 New player starts a survival world
    S01 -> S02 -> S03 -> S04 -> S05
    happy path clicks: 4 (Play, Create New, Create, spawn)
    edge: seed taken/empty, world fails to load, spawn in cave/ocean

F02 Player gets a wooden pickaxe (first tool)
    S05 punch tree -> S05 open inventory (planks) -> S06 craft sticks+crafting table
    -> S05 place table -> S07 craft wooden pickaxe -> S05 mine stone
    happy path actions: ~9
    edge: inventory full, no wood nearby, table destroyed before use

F03 Survive the night
    S05 gather + build shelter before dark -> mobs spawn outside
    -> S05/S06 manage food/health -> dawn
    edge: caught outside, low hunger, fall damage

F04 Die and respawn
    S05 (health 0) -> S10 -> S05 at spawn point
    edge: keep inventory (gamemode), items drop on death

F05 Save and quit, resume later
    S05 -> S08 -> S02 (world saved to disk) -> later S02 -> S04 -> S05
    edge: crash mid-save, disk full
```

## Components

| component | variants | states | used on |
| --- | --- | --- | --- |
| Block | solid, transparent, fluid, plant | solid/air/water states | world |
| Hotbar | 9 slots, selected highlight | empty, filled, selected | S05 |
| Health/hearts + hunger bar | — | full, half, regenerating, starving | S05 |
| Crosshair + block outline | — | target block highlighted / none | S05 |
| Item slot / grid | inventory, crafting, chest | empty, filled, hover, drag | S06, S07 |
| Button (UI) | world menu, pause, settings | default, hover, pressed | S01–S10 |
| Slider | render distance, volume, FOV | default, dragging | S09 |
| Text input | world name, seed, chat | empty, typing, too long | S03, S11 |
| Mobs (HUD-adjacent) | zombie, skeleton, creeper, animal | idle, chase, attack, dead | S05 |

## Inferred data model

```
World      seed, gamemode, time_of_day, spawn_pos, player_state
           evidence: S03 form, F05 save/resume, S10 respawn
           confidence: high

Chunk      pos(x,z), block_ids[16×h×16], light[...], dirty_flag
           evidence: S05 streaming terrain, render distance in S09
           confidence: high (16-wide chunks are publicly documented)

Block      id, name, hardness, tool_type, drops, texture_top/side/bottom, solidity, transparency
           evidence: minecraft.wiki block pages, S07 recipes
           confidence: high

Player     pos, vel, yaw/pitch, health, hunger, inventory(36 slots), xp
           evidence: S05 HUD, S06 grid, S10 death
           confidence: high

Item       id, name, max_stack(1/16/64), tool_props(damage, durability, tier), is_block
           evidence: S06/S07, wiki item pages
           confidence: high

Recipe     pattern(1×1..3×3), ingredient ids, result(item, count)
           evidence: S07 crafting table, wiki recipe list
           confidence: high

Entity     id, type(mob/player/item_drop), pos, vel, health, ai_state
           evidence: S05 mobs + dropped items
           confidence: high

SaveFile   version, world data, chunks (region files), player data
           evidence: F05 save/resume
           confidence: medium (format is our own, not Mojang's)
```

Relationships: World 1-n Chunk, World 1-n Entity, Player 1-1 Inventory, Inventory 1-n Slot, Slot n-1 Item, Recipe n-m Item, Chunk 1-n Block.

## Feature matrix

See `features.csv`. Must: 14, should: 9, could: 7, skip: 5.

## Out of scope (cannot or should not be cloned)

- Minecraft name, logo, textures, sounds, models — all trademarked/copyrighted. Blockforge uses own name + own/procedural assets.
- Mojang account system, Realms, marketplace, servers list.
- The Anvil/region save format — we design our own format.
- Minecraft's code (obfuscated Java) — never read or copy.
- Licensed music discs / soundtrack (C418) — own music or silence.
- Mods marketplace, skin marketplace, Bedrock crossplay.

## Size

Screens 11, flows 5, entities 7. Hard parts: chunk meshing performance, lighting (smooth light + day/night), save format, mob AI. Size: **L** (a quarter for full plan; M for phases 0–6 playable).

## Tech stack (decided)

- Rust 1.98 + **Bevy 0.20** (latest stable, 2026-10-08)
- `block-mesh` 0.2 (greedy meshing) — own chunk system (not bevy_voxel_world, so the engine is fully ours)
- `noise` 0.9 (terrain), `rand` (decorations)
- Own AABB collision + raycast (no physics dep for v1)
- Own flycam/player controller (bevy_flycam lags Bevy versions)
