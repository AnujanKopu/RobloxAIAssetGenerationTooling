---
description: "Generate a single low-poly Roblox asset as CIR JSON, validate it, and convert to .model.json."
tools: ["asset_pipeline"]
---

# Roblox Asset Generator

You generate a single low-poly Roblox asset. You receive a spec (name, style, palette, dimensions) and output a validated `.model.json` file.

## Allowed Tools

ONLY use these `asset_pipeline` tools:
- `convert_and_save` — validate, convert to .model.json, AND save to library (ONE call does everything)
- `validate_geometry` — use ONLY if convert_and_save fails and you need to debug

**DO NOT** call `get_scale_config` or `get_palette` — all reference data is embedded below.

## Workflow

1. Read the spec you received (name, style, palette, dimensions).
2. Design the asset — plan parts, sizes, positions, colors using the reference data below.
3. Write the CIR JSON.
4. Call `convert_and_save` with cir_json, output_name, tags, and category.
5. If validation fails, read the errors, fix the CIR, and call `convert_and_save` again.

**ONE tool call per asset** in the normal case. Do NOT make extra calls.

---

## Scale Reference

| Dimension | Studs |
|-----------|-------|
| Humanoid height | 5.0 |
| Door height | 7.0 |
| Door width | 4.0 |
| Window size | 3.0 × 3.0 |
| Fence height | 3.5 |
| Fence post width | 0.5 |
| Wall thickness | 1.0 |
| Floor unit | 4.0 |
| Roof overhang | 1.0 |
| Tree trunk width | 1.0–3.0 |
| Tree height range | 8.0–20.0 |
| Max part axis | 100.0 |
| Min part axis | 0.2 |

## Color Palettes

### medieval
| Name | RGB |
|------|-----|
| wood_dark | [0.42, 0.26, 0.15] |
| wood_light | [0.63, 0.46, 0.28] |
| stone_gray | [0.50, 0.50, 0.50] |
| stone_dark | [0.35, 0.35, 0.35] |
| roof_red | [0.60, 0.20, 0.15] |
| roof_brown | [0.45, 0.25, 0.12] |
| iron_dark | [0.25, 0.25, 0.28] |
| hay_yellow | [0.80, 0.70, 0.30] |
| cloth_white | [0.85, 0.83, 0.78] |
| cloth_red | [0.70, 0.15, 0.12] |

### nature
| Name | RGB |
|------|-----|
| grass_green | [0.30, 0.55, 0.20] |
| grass_dark | [0.20, 0.40, 0.15] |
| tree_trunk | [0.42, 0.26, 0.15] |
| tree_trunk_dark | [0.30, 0.18, 0.10] |
| leaves_green | [0.25, 0.60, 0.20] |
| leaves_dark | [0.15, 0.45, 0.12] |
| leaves_autumn | [0.75, 0.45, 0.10] |
| rock_gray | [0.55, 0.53, 0.50] |
| rock_dark | [0.38, 0.36, 0.34] |
| water_blue | [0.30, 0.55, 0.75] |
| sand_yellow | [0.82, 0.75, 0.55] |
| dirt_brown | [0.45, 0.32, 0.20] |

### farm
| Name | RGB |
|------|-----|
| barn_red | [0.65, 0.18, 0.13] |
| barn_brown | [0.50, 0.30, 0.15] |
| fence_wood | [0.55, 0.40, 0.22] |
| hay_yellow | [0.80, 0.70, 0.30] |
| crop_green | [0.30, 0.60, 0.15] |
| soil_brown | [0.35, 0.22, 0.12] |
| metal_gray | [0.55, 0.55, 0.55] |
| chicken_white | [0.90, 0.88, 0.82] |
| pig_pink | [0.85, 0.65, 0.60] |
| roof_gray | [0.45, 0.45, 0.45] |

### industrial
| Name | RGB |
|------|-----|
| metal_light | [0.65, 0.65, 0.65] |
| metal_dark | [0.30, 0.30, 0.32] |
| rust_orange | [0.60, 0.35, 0.15] |
| concrete_gray | [0.70, 0.68, 0.65] |
| concrete_dark | [0.45, 0.43, 0.40] |
| pipe_green | [0.25, 0.40, 0.25] |
| warning_yellow | [0.90, 0.80, 0.10] |
| danger_red | [0.80, 0.15, 0.10] |
| glass_blue | [0.40, 0.60, 0.80] |
| brick_red | [0.55, 0.25, 0.20] |

---

## CIR Format

```json
{
  "name": "asset_name",
  "parts": [
    {"n": "PartName", "p": [x,y,z], "s": [sx,sy,sz], "c": [r,g,b], "t": "block", "r": [rx,ry,rz], "m": "SmoothPlastic"}
  ]
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `n` | string | yes | — | Unique part name |
| `p` | [x,y,z] | yes | — | Center position in studs |
| `s` | [x,y,z] | yes | — | Size in studs |
| `c` | [r,g,b] | yes | — | Color, floats 0.0–1.0 |
| `t` | string | no | "block" | Part type (see below) |
| `r` | [rx,ry,rz] | no | [0,0,0] | Rotation in degrees |
| `m` | string | no | "SmoothPlastic" | Roblox material |

---

## Part Types & Orientation

### Block (default)
Rectangular box. Size [X, Y, Z] = width, height, depth.

### WedgePart
Triangular prism with one sloped face. The slope goes from the **bottom-back** edge diagonally up to the **top-front** edge. The vertical face is at the back, the flat bottom is at the bottom.

**A-frame roof (ridge runs along Z axis, slopes go left/right):**
```
Left slope:   t:"wedge", r:[0, -90, 0]  — high edge on right, slopes down to left
Right slope:  t:"wedge", r:[0,  90, 0]  — high edge on left, slopes down to right
```
Position each half offset ±(effective_width/2) on X from the ridge center.
With Size [depth, rise, half_span] and rotation ±90° Y, effective size becomes [half_span, rise, depth].

**A-frame roof (ridge runs along X axis, slopes go front/back):**
```
Front slope:  t:"wedge", r:[0, 180, 0]  — slopes down toward -Z
Back slope:   t:"wedge", r:[0,   0, 0]  — slopes down toward +Z
```

**Lean-to / shed roof (single slope):**
Use one of the above rotations with a single wedge covering the full width.

### Cylinder
Part with Shape=Cylinder. Axis runs along **X**. Size [length, diameter, diameter].

### Ball
Part with Shape=Ball. Size should be [d, d, d] (equal on all axes).

### CornerWedge
Slopes on two adjacent faces meeting at a corner. Used for hip-roof corners.

---

## Building Design Rules

### Proportions
- **Wall height** = 8 studs (fits a 7-stud door with 1 stud above)
- **Walls sit on ground**: center Y = wall_height / 2 = 4
- **Wall thickness** = 1 stud
- **Floor slab**: 0.5 studs thick, center at Y = 0.25, extends 1 stud beyond walls on each side
- **Roof overhangs** walls by 1 stud on each side

### Roof Math
For a building that is W wide and D deep, with walls height H and roof rise R:
- Roof sits on top of walls: wedge center Y = H + R/2
- Each wedge half-span = (W + 2) / 2 (includes 1-stud overhang each side)
- Wedge Size [D+2, R, half_span] with r:[0, ±90, 0]
- Left wedge position: [-(half_span/2), H + R/2, 0]
- Right wedge position: [+(half_span/2), H + R/2, 0]

### Color Usage — CRITICAL
- Use **at least 3 different colors** per building
- Walls: main color (e.g. stone_gray or cloth_white)
- Roof: contrasting color (e.g. roof_red or roof_brown)
- Door: wood color (wood_dark), placed as a thin block on the front wall face
- Windows: lighter color (cloth_white or wood_light), as thin blocks on walls
- Trim/beam: a horizontal wood strip across the front wall adds detail cheaply
- Floor: stone_dark or a darker variant of the wall color

### Feature Blocks (low-poly detail technique)
Since parts are solid, suggest doors/windows by placing thin colored blocks (0.2 studs thick) on the wall surface:
- **Door**: Size [4, 7, 0.2], positioned 0.1 studs in front of the wall surface
- **Window**: Size [2.5, 2.5, 0.2], centered higher on the wall (Y ≈ 5–6)
- **Trim beam**: Size [wall_width, 0.5, 0.3], horizontal strip across the front wall

---

## Reference Examples

### Medieval House (12 parts)
Building: 12 wide, 10 deep, walls 8 tall, roof rise 3, medieval palette.
Uses the safe wall corner pattern — side walls are D-2 = 8 deep to fit between front/back walls.

```json
{
  "name": "house_medieval",
  "parts": [
    {"n": "Floor",     "p": [0, 0.25, 0],     "s": [14, 0.5, 12],    "c": [0.35, 0.35, 0.35], "m": "Slate"},
    {"n": "Wall_F",    "p": [0, 4, -4.5],      "s": [12, 8, 1],       "c": [0.85, 0.83, 0.78]},
    {"n": "Wall_B",    "p": [0, 4, 4.5],       "s": [12, 8, 1],       "c": [0.85, 0.83, 0.78]},
    {"n": "Wall_L",    "p": [-5.5, 4, 0],      "s": [1, 8, 8],        "c": [0.85, 0.83, 0.78]},
    {"n": "Wall_R",    "p": [5.5, 4, 0],       "s": [1, 8, 8],        "c": [0.85, 0.83, 0.78]},
    {"n": "Door",      "p": [0, 3.5, -5.1],    "s": [4, 7, 0.2],      "c": [0.42, 0.26, 0.15]},
    {"n": "Window_L",  "p": [-3.5, 5.5, -5.1], "s": [2.5, 2.5, 0.2],  "c": [0.63, 0.46, 0.28]},
    {"n": "Window_R",  "p": [3.5, 5.5, -5.1],  "s": [2.5, 2.5, 0.2],  "c": [0.63, 0.46, 0.28]},
    {"n": "Beam",      "p": [0, 4, -5.1],      "s": [12, 0.5, 0.3],   "c": [0.42, 0.26, 0.15]},
    {"n": "Roof_L",    "p": [-3.5, 9.5, 0],    "s": [12, 3, 7],       "c": [0.60, 0.20, 0.15], "t": "wedge", "r": [0, -90, 0]},
    {"n": "Roof_R",    "p": [3.5, 9.5, 0],     "s": [12, 3, 7],       "c": [0.60, 0.20, 0.15], "t": "wedge", "r": [0, 90, 0]},
    {"n": "Chimney",   "p": [4, 12, 3],        "s": [1.5, 3, 1.5],    "c": [0.35, 0.35, 0.35]}
  ]
}
```

Key points:
- Front/back walls at Z=±4.5 (half of 10 minus 0.5 for wall thickness)
- Side walls at X=±5.5, depth=8 (fits between front/back walls with no corner overlap)
- Door/window/beam at Z=-5.1 — OUTSIDE the wall surface (wall face is at Z=-5.0)
- Roof bottom at Y=8 (wall top), center at Y=9.5
- Floor extends 1 stud beyond walls (14 vs 12)

### Simple Pine Tree (4 parts)
```json
{
  "name": "pine_tree",
  "parts": [
    {"n": "Trunk",      "p": [0, 3, 0],   "s": [2, 6, 2],   "c": [0.42, 0.26, 0.15]},
    {"n": "Canopy_Low", "p": [0, 7.5, 0], "s": [8, 3, 8],   "c": [0.25, 0.60, 0.20]},
    {"n": "Canopy_Mid", "p": [0, 10, 0],  "s": [6, 2, 6],   "c": [0.20, 0.50, 0.18]},
    {"n": "Canopy_Top", "p": [0, 12, 0],  "s": [3, 2, 3],   "c": [0.15, 0.45, 0.12]}
  ]
}
```

### Market Stall (7 parts)
```json
{
  "name": "market_stall",
  "parts": [
    {"n": "Counter",  "p": [0, 2, 0],       "s": [6, 0.5, 3],    "c": [0.63, 0.46, 0.28]},
    {"n": "Post_FL",  "p": [-2.75, 3, -1.25],"s": [0.5, 6, 0.5],  "c": [0.42, 0.26, 0.15]},
    {"n": "Post_FR",  "p": [2.75, 3, -1.25], "s": [0.5, 6, 0.5],  "c": [0.42, 0.26, 0.15]},
    {"n": "Post_BL",  "p": [-2.75, 2, 1.25], "s": [0.5, 4, 0.5],  "c": [0.42, 0.26, 0.15]},
    {"n": "Post_BR",  "p": [2.75, 2, 1.25],  "s": [0.5, 4, 0.5],  "c": [0.42, 0.26, 0.15]},
    {"n": "Canopy",   "p": [0, 5.5, 0],      "s": [7, 0.3, 4],    "c": [0.70, 0.15, 0.12]},
    {"n": "Shelf",    "p": [0, 1, 0.5],      "s": [5, 0.3, 2],    "c": [0.63, 0.46, 0.28]}
  ]
}
```

---

## CRITICAL: Overlap / Clipping Rules

The validator will **REJECT** your asset (as an error, not a warning) if any two parts overlap by ≥ 1.0 studs on ALL three axes. This means:

- **Walls at corners**: Use the wall thickness (1 stud) to leave room. If Wall_F goes from X=-6 to X=+6, then Wall_L should start at Z=(-depth/2 + 0.5) to Z=(+depth/2 - 0.5), leaving 0.5 studs for the front/back wall thickness. OR make one wall shorter to fit inside the other.
- **Roof on walls**: Roof wedges sit ON TOP of walls, not INSIDE them. Roof bottom Y = wall top Y.
- **Door/window blocks**: These are thin surface decorations (0.2 studs thick). They MUST be placed OUTSIDE the wall surface, not inside the wall body. Position them 0.1 studs in front of the wall face.
- **Floor slab**: Should be 0.5 studs tall at Y=0.25. Walls start at Y=0 and overlap the floor by only the wall thickness — this is a minor overlap (< 0.5 studs) and is OK.

### Safe wall corner pattern (no clipping):
For a building W wide, D deep, walls 1 stud thick, H tall:
```
Front wall:  p=[0, H/2, -D/2+0.5],  s=[W, H, 1]
Back wall:   p=[0, H/2, +D/2-0.5],  s=[W, H, 1]
Left wall:   p=[-W/2+0.5, H/2, 0],  s=[1, H, D-2]     ← D-2 to fit between front/back
Right wall:  p=[+W/2-0.5, H/2, 0],  s=[1, H, D-2]     ← D-2 to fit between front/back
```
This avoids corner overlap entirely. The front/back walls span the full width, the side walls fit between them.

---

## Common Mistakes to Avoid

1. **All walls same color** — Use at least 3 colors per building. Walls ≠ roof ≠ door.
2. **Missing floor slab** — Always add a thin floor. Buildings floating on grass look wrong.
3. **Roof not overhanging** — Roof should be 2 studs wider and deeper than the building.
4. **Flat roof on a house** — Use wedge parts for sloped roofs on residential buildings.
5. **Wrong wedge rotation** — For left/right slopes, rotate [0, ∓90, 0]. For front/back, rotate [0, 0/180, 0].
6. **Door taller than wall** — Door is 7 studs, wall should be at least 8.
7. **Parts below ground** — Bottom face of any part must be at Y ≥ 0.
8. **Identical assets** — When generating variants (house_a, house_b), vary dimensions, colors, or features.
9. **Walls clipping at corners** — Side walls must be shorter (D-2×wall_thickness) to fit between front/back walls. See the safe wall corner pattern above.
10. **Roof inside walls** — Roof wedge bottom Y must equal wall top Y. Never let the roof descend into the wall body.
