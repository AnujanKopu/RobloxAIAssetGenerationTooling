use serde_json::Value;
use std::path::Path;

pub struct SceneObject {
    pub name: String,
    pub center: [f64; 3],
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
    pub part_count: usize,
}

pub struct OpenArea {
    pub min: [f64; 2],
    pub max: [f64; 2],
    pub size: [f64; 2],
}

pub struct SceneContext {
    pub objects: Vec<SceneObject>,
    pub open_areas: Vec<OpenArea>,
}

/// Read all .model.json files from the generated directory and build a scene context
pub fn read_scene(generated_dir: &Path) -> anyhow::Result<SceneContext> {
    let mut objects = Vec::new();

    if !generated_dir.exists() {
        return Ok(SceneContext {
            objects,
            open_areas: Vec::new(),
        });
    }

    read_dir_recursive(generated_dir, &mut objects)?;
    let open_areas = find_open_areas(&objects);

    Ok(SceneContext {
        objects,
        open_areas,
    })
}

fn read_dir_recursive(dir: &Path, objects: &mut Vec<SceneObject>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            read_dir_recursive(&path, objects)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".model.json") {
                if let Ok(obj) = parse_model_json(&path) {
                    objects.push(obj);
                }
            }
        }
    }
    Ok(())
}

fn parse_model_json(path: &Path) -> anyhow::Result<SceneObject> {
    let content = std::fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&content)?;

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .replace(".model", "")
        .to_string();

    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid model JSON"))?;

    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    let mut part_count = 0;

    collect_parts(obj, &mut min, &mut max, &mut part_count);

    if part_count == 0 {
        return Err(anyhow::anyhow!("No parts found in model"));
    }

    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];

    Ok(SceneObject {
        name,
        center,
        bbox_min: min,
        bbox_max: max,
        part_count,
    })
}

/// Recursively collect part positions/sizes from a Rojo model JSON tree
fn collect_parts(
    obj: &serde_json::Map<String, Value>,
    min: &mut [f64; 3],
    max: &mut [f64; 3],
    part_count: &mut usize,
) {
    // Check if this node itself has properties with Position and Size
    if let Some(props) = obj.get("properties").and_then(|p| p.as_object()) {
        if let (Some(pos_arr), Some(size_arr)) = (
            props.get("Position").and_then(|p| p.as_array()),
            props.get("Size").and_then(|s| s.as_array()),
        ) {
            let pos: Vec<f64> = pos_arr.iter().filter_map(|v| v.as_f64()).collect();
            let size: Vec<f64> = size_arr.iter().filter_map(|v| v.as_f64()).collect();
            if pos.len() == 3 && size.len() == 3 {
                for axis in 0..3 {
                    min[axis] = min[axis].min(pos[axis] - size[axis] / 2.0);
                    max[axis] = max[axis].max(pos[axis] + size[axis] / 2.0);
                }
                *part_count += 1;
            }
        }
    }

    // Recurse into children array
    if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
        for child in children {
            if let Some(child_obj) = child.as_object() {
                collect_parts(child_obj, min, max, part_count);
            }
        }
    }
}

fn find_open_areas(objects: &[SceneObject]) -> Vec<OpenArea> {
    if objects.is_empty() {
        return vec![OpenArea {
            min: [-100.0, -100.0],
            max: [100.0, 100.0],
            size: [200.0, 200.0],
        }];
    }

    let mut scene_min_x = f64::MAX;
    let mut scene_max_x = f64::MIN;
    let mut scene_min_z = f64::MAX;
    let mut scene_max_z = f64::MIN;

    for obj in objects {
        scene_min_x = scene_min_x.min(obj.bbox_min[0]);
        scene_max_x = scene_max_x.max(obj.bbox_max[0]);
        scene_min_z = scene_min_z.min(obj.bbox_min[2]);
        scene_max_z = scene_max_z.max(obj.bbox_max[2]);
    }

    let margin = 50.0;
    let x_start = scene_min_x - margin;
    let x_end = scene_max_x + margin;
    let z_start = scene_min_z - margin;
    let z_end = scene_max_z + margin;

    let mut areas = Vec::new();

    // Left of scene
    if scene_min_x - x_start > 10.0 {
        areas.push(OpenArea {
            min: [x_start, z_start],
            max: [scene_min_x - 2.0, z_end],
            size: [scene_min_x - 2.0 - x_start, z_end - z_start],
        });
    }

    // Right of scene
    if x_end - scene_max_x > 10.0 {
        areas.push(OpenArea {
            min: [scene_max_x + 2.0, z_start],
            max: [x_end, z_end],
            size: [x_end - scene_max_x - 2.0, z_end - z_start],
        });
    }

    // Front of scene
    if scene_min_z - z_start > 10.0 {
        areas.push(OpenArea {
            min: [scene_min_x, z_start],
            max: [scene_max_x, scene_min_z - 2.0],
            size: [scene_max_x - scene_min_x, scene_min_z - 2.0 - z_start],
        });
    }

    // Behind scene
    if z_end - scene_max_z > 10.0 {
        areas.push(OpenArea {
            min: [scene_min_x, scene_max_z + 2.0],
            max: [scene_max_x, z_end],
            size: [scene_max_x - scene_min_x, z_end - scene_max_z - 2.0],
        });
    }

    areas
}

/// Format scene context as human-readable text for Claude
pub fn format_scene_context(ctx: &SceneContext) -> String {
    let mut out = String::new();

    if ctx.objects.is_empty() {
        return "Scene is empty. No generated assets found.\nEntire area is available for placement."
            .to_string();
    }

    out.push_str("Scene objects:\n");
    for obj in &ctx.objects {
        let size_x = obj.bbox_max[0] - obj.bbox_min[0];
        let size_y = obj.bbox_max[1] - obj.bbox_min[1];
        let size_z = obj.bbox_max[2] - obj.bbox_min[2];
        out.push_str(&format!(
            "  {} — center ({:.0},{:.0},{:.0}), bbox {:.0}x{:.0}x{:.0} studs, {} parts\n",
            obj.name, obj.center[0], obj.center[1], obj.center[2], size_x, size_y, size_z,
            obj.part_count
        ));
    }

    out.push_str("\nOpen areas (XZ plane):\n");
    for area in &ctx.open_areas {
        out.push_str(&format!(
            "  ({:.0},{:.0}) to ({:.0},{:.0}) — {:.0}x{:.0} studs\n",
            area.min[0], area.min[1], area.max[0], area.max[1], area.size[0], area.size[1]
        ));
    }

    out
}
