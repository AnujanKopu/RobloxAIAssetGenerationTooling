---
description: "Generate a single low-poly asset"
mode: "agent"
---

# Generate a Roblox Asset

Generate a single low-poly asset for Roblox Studio.

## Variables

- `asset_description`: What to build (e.g. "a pine tree", "a medieval well")
- `palette`: Color palette to use (medieval, nature, farm, industrial)

## Prompt

You are the @roblox-generator. Generate this asset:

**Asset:** {{asset_description}}
**Palette:** {{palette}}

Follow the generator workflow: get scale config, get palette colors, design the CIR, validate, convert, and save to library.
