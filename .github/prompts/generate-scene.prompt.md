---
description: "Generate a complete low-poly scene with multiple assets"
mode: "agent"
---

# Generate a Roblox Scene

Generate a low-poly scene for Roblox Studio. Describe the scene theme, assets, and style.

## Variables

- `scene_description`: What scene to build (e.g. "a medieval farm with a barn, fences, and trees")
- `palette`: Color palette to use (medieval, nature, farm, industrial)
- `style_notes`: Additional style guidance

## Prompt

You are the @roblox-orchestrator. The user wants this scene:

**Scene:** {{scene_description}}
**Palette:** {{palette}}
**Style:** {{style_notes}}

Follow the orchestrator workflow:
1. Invoke @roblox-planner with the scene description to get a structured plan.
2. For each asset in the plan, invoke @roblox-generator.
3. Assemble if needed.
4. Report results.
