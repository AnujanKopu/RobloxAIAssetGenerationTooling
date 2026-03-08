Generate a single low-poly Roblox asset.

Asset description: $ARGUMENTS

## Workflow

1. Design the asset — plan parts, sizes, positions, colors.
2. Write the CIR JSON.
3. Call `convert_and_save` with cir_json, output_name, tags, and category.
4. If validation fails, read the errors, fix the CIR, and retry (max 2 attempts).

**ONE tool call per asset** in the normal case.

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
wood_dark [0.42,0.26,0.15], wood_light [0.63,0.46,0.28], stone_gray [0.50,0.50,0.50], stone_dark [0.35,0.35,0.35], roof_red [0.60,0.20,0.15], roof_brown [0.45,0.25,0.12], iron_dark [0.25,0.25,0.28], hay_yellow [0.80,0.70,0.30], cloth_white [0.85,0.83,0.78], cloth_red [0.70,0.15,0.12]

### nature
grass_green [0.30,0.55,0.20], grass_dark [0.20,0.40,0.15], tree_trunk [0.42,0.26,0.15], tree_trunk_dark [0.30,0.18,0.10], leaves_green [0.25,0.60,0.20], leaves_dark [0.15,0.45,0.12], leaves_autumn [0.75,0.45,0.10], rock_gray [0.55,0.53,0.50], rock_dark [0.38,0.36,0.34], water_blue [0.30,0.55,0.75], sand_yellow [0.82,0.75,0.55], dirt_brown [0.45,0.32,0.20]

### farm
barn_red [0.65,0.18,0.13], barn_brown [0.50,0.30,0.15], fence_wood [0.55,0.40,0.22], hay_yellow [0.80,0.70,0.30], crop_green [0.30,0.60,0.15], soil_brown [0.35,0.22,0.12], metal_gray [0.55,0.55,0.55], chicken_white [0.90,0.88,0.82], pig_pink [0.85,0.65,0.60], roof_gray [0.45,0.45,0.45]

### industrial
metal_light [0.65,0.65,0.65], metal_dark [0.30,0.30,0.32], rust_orange [0.60,0.35,0.15], concrete_gray [0.70,0.68,0.65], concrete_dark [0.45,0.43,0.40], pipe_green [0.25,0.40,0.25], warning_yellow [0.90,0.80,0.10], danger_red [0.80,0.15,0.10], glass_blue [0.40,0.60,0.80], brick_red [0.55,0.25,0.20]

## CIR Format

```json
{
  "name": "asset_name",
  "parts": [
    {"n": "PartName", "p": [x,y,z], "s": [sx,sy,sz], "c": [r,g,b], "t": "block", "r": [rx,ry,rz], "m": "SmoothPlastic"}
  ]
}
```

## Part Types

- **block** (default): Box. Size [width, height, depth].
- **wedge**: Slope from bottom-back to top-front. A-frame roof: left slope r:[0,-90,0], right slope r:[0,90,0].
- **cylinder**: Axis along X. Size [length, diameter, diameter].
- **ball**: Sphere. Size [d, d, d].
- **corner_wedge**: Hip-roof corners.

## Building Rules

- Wall height = 8 studs, center Y = 4
- Floor slab: 0.5 studs at Y=0.25, extends 1 stud beyond walls
- Roof overhangs 1 stud, uses wedge parts
- At least 3 colors per building
- Doors (4×7×0.2) and windows (2.5×2.5×0.2) are thin blocks OUTSIDE wall surface
- Safe wall corner pattern: front/back span full width, side walls are D-2 to fit between them
- Overlap ≥ 1 stud = error (blocks conversion)

## Reference: Medieval House (12 parts)
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
