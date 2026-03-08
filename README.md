<div align="center">

# roblox-asset-pipeline

**AI-powered low-poly 3D asset generation for Roblox Studio**

Describe what you want. Get optimized 3D assets synced directly into your game.

</div>

---

Generate low-poly Roblox assets by chatting with an AI agent. Type a prompt, and the pipeline validates, converts, and drops production-ready `.model.json` files straight into Studio via Rojo.

Works with **GitHub Copilot**, **Cursor**, and **Claude Code** out of the box.

```
# GitHub Copilot (VS Code)
@roblox-orchestrator build a small medieval village

# Cursor
> build a small medieval village          (auto-picks orchestrator rules)

# Claude Code
/generate-scene a small medieval village
```

---

## Features

- **Natural language to 3D** — describe a barn, get a properly built low-poly barn with walls, roof, and doors
- **Geometry validation** — every asset is checked before output: no clipping, no floating parts, no scale violations
- **Rotation-aware bounds** — wedge roofs and angled parts get accurate bounding boxes, not axis-aligned approximations
- **Asset library** — generated assets are saved and reused automatically across scenes
- **Scene assembly** — place multiple assets together with automatic inter-asset collision detection
- **Consistent palettes** — 4 named color palettes (medieval, farm, nature, industrial) keep your game visually coherent
- **Fast** — the primary `convert_and_save` tool does validate + convert + save in a single MCP call

---

## Requirements

- **Editor** (any one):
  - [VS Code](https://code.visualstudio.com) with [GitHub Copilot](https://github.com/features/copilot)
  - [Cursor](https://cursor.sh)
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- [Rust](https://rustup.rs) — to build the MCP server
- [Rojo](https://rojo.space) + the Roblox Studio plugin — to sync assets into Studio

---

## Getting Started

### 1. Drop `pipeline/` into your Roblox project

Clone this repo or copy the `pipeline/` folder into your existing Roblox project root.

```
your-game/
  pipeline/         ← this repo
  src/
  default.project.json
```

### 2. Build the MCP server

```bash
cd pipeline/mcp-server
cargo build
```

Binary lands at `pipeline/mcp-server/target/debug/roblox-asset-pipeline.exe`.

### 3. Point Rojo at the generated assets

In your `default.project.json`, add the generated assets folder:

```json
{
  "name": "your-game",
  "tree": {
    "$className": "DataModel",
    "GeneratedAssets": {
      "$path": "pipeline/assets/generated"
    }
  }
}
```

### 4. Configure your editor

Copy the MCP config for your editor into your project:

| Editor | Config file | Copy from |
|--------|-------------|----------|
| VS Code (Copilot) | `.vscode/mcp.json` | This repo's `.vscode/mcp.json` |
| Cursor | `.cursor/mcp.json` | This repo's `.cursor/mcp.json` |
| Claude Code | `.mcp.json` | This repo's `.mcp.json` |

Also copy the agent/rules files:

| Editor | Files to copy |
|--------|---------------|
| VS Code (Copilot) | `.github/agents/`, `.github/instructions/`, `.github/copilot-instructions.md` |
| Cursor | `.cursor/rules/`, `.cursorrules` |
| Claude Code | `CLAUDE.md`, `.claude/commands/` |

Update the binary path in the MCP config if your project layout differs.

### 5. Start Rojo and generate

```bash
rojo serve
```

Then use your editor's AI chat:

**GitHub Copilot:**
```
@roblox-orchestrator make a medieval village
@roblox-generator create a tall oak tree
```

**Cursor:**
```
make a medieval village
create a tall oak tree
```
Cursor's agent mode auto-selects the right rules based on your prompt.

**Claude Code:**
```
/generate-scene a medieval village
/generate-asset a tall oak tree
```

Assets appear in Studio automatically as they generate.

---

## Usage

### Agents / Commands

| Role | Copilot | Cursor | Claude Code |
|------|---------|--------|-------------|
| Full scene | `@roblox-orchestrator <prompt>` | Auto (orchestrator rules) | `/generate-scene <prompt>` |
| Single asset | `@roblox-generator <prompt>` | Auto (generator rules) | `/generate-asset <prompt>` |
| Plan only | `@roblox-planner <prompt>` | Auto (planner rules) | — |

### Examples

```
# Scene generation
make a haunted forest with dead trees and a broken fence
build a port with docks, crates, and a lighthouse

# Single asset
create a stone well with a wooden roof
generate a market stall with an awning
```

---

## How It Works

Assets go through a strict pipeline:

```
prompt
  └─ Planner      picks assets, positions, palette
       └─ Generator  writes CIR JSON for each asset
            └─ Validator   checks geometry (clipping, scale, overlap)
                 └─ Converter  CIR → .model.json
                      └─ Rojo      .model.json → Roblox Studio
```

### CIR Format

Assets are authored in **CIR (Compact Intermediate Representation)** — a minimal JSON format designed to be token-efficient for AI generation:

```json
{
  "name": "pine_tree",
  "parts": [
    { "n": "Trunk",      "p": [0, 3,   0], "s": [2, 6, 2], "c": [0.42, 0.26, 0.15] },
    { "n": "Canopy_Low", "p": [0, 7.5, 0], "s": [8, 3, 8], "c": [0.25, 0.60, 0.20] },
    { "n": "Canopy_Top", "p": [0, 11,  0], "s": [4, 3, 4], "c": [0.20, 0.50, 0.18] }
  ]
}
```

Full spec in [`.github/instructions/cir-format.instructions.md`](.github/instructions/cir-format.instructions.md).

### Part Budget

| Complexity | Parts |
|------------|-------|
| Simple (rock, bale) | 3–8 |
| Medium (tree, fence) | 8–15 |
| Complex (building) | 15–25 |
| Hard max | 30 |

---

## Asset Library

Every generated asset is saved to `pipeline/assets/library/` with searchable tags. On future runs the pipeline searches the library first and reuses existing assets instead of regenerating.

```
pipeline/assets/library/
  index.json          ← search index
  structures/         ← buildings, fences, wells
  nature/             ← trees, rocks, plants
  farm/               ← crops, troughs, bales
  decorations/        ← barrels, crates, carts
```

---

## Color Palettes

Defined in `pipeline/config/palette.json`:

| Palette | Use for |
|---------|---------|
| `medieval` | Castles, villages, dungeons |
| `farm` | Barns, crops, fences, silos |
| `nature` | Forests, parks, wilderness |
| `industrial` | Factories, ports, warehouses |

---

## Project Structure

```
pipeline/
  mcp-server/         Rust MCP server (9 tools)
    src/
      main.rs         Tool dispatch + MCP protocol
      cir.rs          CIR data types
      converter.rs    CIR → .model.json
      validator.rs    Geometry validation
      assembler.rs    Multi-asset scene assembly
      asset_library.rs  Library CRUD + fuzzy search
    Cargo.toml
  config/
    scale.json        Canonical game dimensions
    palette.json      Named color palettes
  assets/
    library/          Reusable CIR assets (committed)
    generated/        Output .model.json files (gitignored)

.github/                        GitHub Copilot agents
  agents/
  instructions/
  copilot-instructions.md
.cursor/                        Cursor rules + MCP config
  rules/
  mcp.json
.claude/                        Claude Code commands
  commands/
.vscode/                        VS Code MCP config
  mcp.json
.mcp.json                       Claude Code MCP config
CLAUDE.md                       Claude Code project context
.cursorrules                    Cursor project context
```

---

## License

MIT
