Generate a complete low-poly Roblox scene with multiple assets.

Scene description: $ARGUMENTS

## Workflow

1. **Plan** — Call `read_scene_context` to see what exists. Call `search_assets` to find reusable library assets. Plan the full asset list with positions.
2. **Generate** — For each asset, design CIR JSON and call `convert_and_save`.
3. **Assemble** — Use `assemble_scene` to merge all assets into one scene `.model.json`.
4. **Report** — Summarize asset count, part totals, and any warnings.

## Planning Rules

- Choose a single palette for visual consistency (medieval, nature, farm, or industrial)
- Space buildings 15–20 studs apart for walkable paths
- Reuse library assets when possible (action: "reuse" with library_ref)
- Part estimates: simple 3–8, medium 8–15, complex 15–25

## Scale Reference

| Dimension | Studs |
|-----------|-------|
| Humanoid height | 5.0 |
| Door height | 7.0 |
| Door width | 4.0 |
| Fence height | 3.5 |
| Wall thickness | 1.0 |
| Tree height range | 8.0–20.0 |

## Available Palettes

- **medieval**: wood_dark [0.42,0.26,0.15], wood_light [0.63,0.46,0.28], stone_gray [0.50,0.50,0.50], stone_dark [0.35,0.35,0.35], roof_red [0.60,0.20,0.15], roof_brown [0.45,0.25,0.12], iron_dark [0.25,0.25,0.28], hay_yellow [0.80,0.70,0.30], cloth_white [0.85,0.83,0.78], cloth_red [0.70,0.15,0.12]
- **nature**: grass_green [0.30,0.55,0.20], tree_trunk [0.42,0.26,0.15], leaves_green [0.25,0.60,0.20], leaves_dark [0.15,0.45,0.12], rock_gray [0.55,0.53,0.50], water_blue [0.30,0.55,0.75], sand_yellow [0.82,0.75,0.55], dirt_brown [0.45,0.32,0.20]
- **farm**: barn_red [0.65,0.18,0.13], barn_brown [0.50,0.30,0.15], fence_wood [0.55,0.40,0.22], hay_yellow [0.80,0.70,0.30], crop_green [0.30,0.60,0.15], soil_brown [0.35,0.22,0.12], metal_gray [0.55,0.55,0.55], roof_gray [0.45,0.45,0.45]
- **industrial**: metal_light [0.65,0.65,0.65], metal_dark [0.30,0.30,0.32], rust_orange [0.60,0.35,0.15], concrete_gray [0.70,0.68,0.65], warning_yellow [0.90,0.80,0.10], danger_red [0.80,0.15,0.10], glass_blue [0.40,0.60,0.80], brick_red [0.55,0.25,0.20]

## Generation Rules

For each asset, follow these rules:
- Use CIR format with `convert_and_save` (ONE call per asset)
- Use at least 3 colors per building
- Always add a floor slab (0.5 studs at Y=0.25)
- Roof overhangs walls by 1 stud, use WedgeParts for sloped roofs
- Doors/windows are thin blocks (0.2 studs) placed OUTSIDE the wall surface
- Side walls must be D-2 to fit between front/back walls (safe corner pattern)
- If validation fails, fix errors and retry (max 2 retries per asset)
