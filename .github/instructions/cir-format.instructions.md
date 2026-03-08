---
applyTo: "**"
---

# CIR (Compact Intermediate Representation) Format

CIR is the token-efficient JSON format for describing low-poly Roblox assets. It is converted to Rojo `.model.json` by the `asset_pipeline` MCP server.

## Schema

```json
{
  "name": "asset_name",
  "primary_part": "optional_part_name",
  "parts": [
    {
      "n": "PartName",
      "p": [x, y, z],
      "s": [sx, sy, sz],
      "c": [r, g, b],
      "t": "block",
      "r": [rx, ry, rz],
      "m": "SmoothPlastic"
    }
  ]
}
```

## Fields

| Field | Required | Type | Default | Description |
|-------|----------|------|---------|-------------|
| `name` | yes | string | — | Model name |
| `primary_part` | no | string | null | PrimaryPart reference by name |
| `parts` | yes | array | — | Array of CirPart objects |

### CirPart Fields

| Field | Required | Type | Default | Description |
|-------|----------|------|---------|-------------|
| `n` | yes | string | — | Unique part name |
| `p` | yes | [f64;3] | — | Center position [x, y, z] in studs |
| `s` | yes | [f64;3] | — | Size [x, y, z] in studs |
| `c` | yes | [f64;3] | — | Color [r, g, b] floats 0.0-1.0 |
| `t` | no | string | "block" | Part type |
| `r` | no | [f64;3] | [0,0,0] | Euler rotation in degrees |
| `m` | no | string | "SmoothPlastic" | Roblox material name |

## Part Types → Roblox Classes

| CIR Type | Roblox Class | Notes |
|----------|-------------|-------|
| `block` | Part | Default |
| `wedge` | WedgePart | Slope on one face |
| `corner_wedge` | CornerWedgePart | Slope on corner |
| `cylinder` | Part (Shape=Cylinder) | Rotated along X axis |
| `ball` | Part (Shape=Ball) | Sphere |

## Positioning Rules

- Position `p` is the CENTER of the part, not a corner.
- A part sitting on the ground with height H should have `p[1] = H/2`.
- Two parts stacked: bottom at `p[1] = H1/2`, top at `p[1] = H1 + H2/2`.
- Parts sharing a wall: offset by `(S1/2 + S2/2)` on the shared axis.

## Example: Simple Tree

```json
{
  "name": "pine_tree",
  "parts": [
    {"n": "Trunk", "p": [0, 3, 0], "s": [2, 6, 2], "c": [0.42, 0.26, 0.15]},
    {"n": "Canopy_Low", "p": [0, 7.5, 0], "s": [8, 3, 8], "c": [0.25, 0.60, 0.20]},
    {"n": "Canopy_Mid", "p": [0, 10, 0], "s": [6, 2, 6], "c": [0.20, 0.50, 0.18]},
    {"n": "Canopy_Top", "p": [0, 12, 0], "s": [3, 2, 3], "c": [0.15, 0.45, 0.12]}
  ]
}
```

## Validation Checks

The `validate_geometry` tool checks:
1. **empty_model** — Must have at least one part
2. **underground** — No parts below Y=0 (bottom face)
3. **overlap** — Warns on AABB intersection (touching is OK)
4. **floating** — Warns if part not on ground and not touching another part
5. **scale_violation** — No dimension exceeds max (100 studs)
6. **extreme_dimension** — No dimension below min (0.2 studs)
7. **part_budget** — Warns if over 30 parts (configurable)
8. **palette** — Warns if colors don't match declared palette
9. **name_collision** — No duplicate part names
