# Roblox Low-Poly Asset Pipeline

This project is a standalone AI-driven pipeline for generating low-poly Roblox assets.
The `pipeline/` directory is self-contained and can be dropped into any Roblox project.

## Architecture

- **CIR Format**: Assets are defined as compact JSON (see `.github/instructions/cir-format.instructions.md`)
- **MCP Server**: `pipeline/mcp-server/` — Rust binary providing 9 tools for validate, convert, search, save, assemble
- **Agents**: Three agents — orchestrator, planner, generator — handle scene/asset creation
- **Output**: `.model.json` files written to `pipeline/assets/generated/`, auto-synced to Studio via `rojo serve`

## Key References

- Scale config: `pipeline/config/scale.json` — canonical dimensions (humanoid=5 studs, door=4×7, fence=3.5)
- Color palettes: `pipeline/config/palette.json` — medieval, nature, farm, industrial
- Asset library: `pipeline/assets/library/` — reusable CIR assets with fuzzy search

## Conventions

- Always use CIR format for asset generation, never raw `.model.json`
- Always validate before converting (the convert tool does this automatically)
- Always check the asset library before generating — reuse when possible
- Part budget: 3-8 simple, 8-15 medium, 15-25 complex, max 30
- Colors: always from `get_palette`, never approximate RGB values
- Position Y: center of part, so a part on ground with height H has Y = H/2
