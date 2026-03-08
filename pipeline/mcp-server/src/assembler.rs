use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::asset_library;
use crate::cir::Cir;
use crate::converter;
use crate::validator;

#[derive(Debug, Clone, Deserialize)]
pub struct AssemblySpec {
    pub name: String,
    pub assets: Vec<AssetPlacement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPlacement {
    /// Library asset name reference
    #[serde(rename = "ref", default)]
    pub asset_ref: Option<String>,
    /// Inline CIR (alternative to ref)
    #[serde(default)]
    pub cir: Option<Cir>,
    /// Target position [x, y, z]
    #[serde(default)]
    pub position: [f64; 3],
    /// Target rotation [rx, ry, rz]
    #[serde(default)]
    pub rotation: [f64; 3],
}

#[derive(Debug, Serialize)]
pub struct AssemblyResult {
    pub path: String,
    pub total_parts: usize,
    pub bounding_box_min: [f64; 3],
    pub bounding_box_max: [f64; 3],
    pub collision_warnings: Vec<String>,
}

/// Assemble multiple assets into a single scene .model.json
pub fn assemble_scene(
    spec: &AssemblySpec,
    library_dir: &Path,
    output_path: &Path,
) -> anyhow::Result<AssemblyResult> {
    let mut scene_children: Vec<Value> = Vec::new();
    let mut name_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let mut total_parts = 0;
    let mut scene_min = [f64::MAX; 3];
    let mut scene_max = [f64::MIN; 3];

    // Track placed asset bounding boxes for inter-asset collision detection
    struct PlacedAsset {
        name: String,
        bb_min: [f64; 3],
        bb_max: [f64; 3],
    }
    let mut placed_assets: Vec<PlacedAsset> = Vec::new();
    let mut collision_warnings: Vec<String> = Vec::new();

    for placement in &spec.assets {
        let cir = resolve_cir(placement, library_dir)?;

        // Offset all parts by the placement position and rotation
        let mut offset_cir = cir.clone();
        for part in &mut offset_cir.parts {
            part.p[0] += placement.position[0];
            part.p[1] += placement.position[1];
            part.p[2] += placement.position[2];
            part.r[0] += placement.rotation[0];
            part.r[1] += placement.rotation[1];
            part.r[2] += placement.rotation[2];
        }

        // Compute rotation-aware bounding box for collision detection
        let (bb_min, bb_max) = validator::model_aabb(&offset_cir);

        // Check collision against all previously placed assets
        // Margin of 0.5 studs — overlaps smaller than this are ignored (touching is OK)
        for prev in &placed_assets {
            if validator::aabb_boxes_overlap(&prev.bb_min, &prev.bb_max, &bb_min, &bb_max, 0.5) {
                collision_warnings.push(format!(
                    "Asset '{}' at [{:.0},{:.0},{:.0}] collides with '{}' — bounding boxes overlap by more than 0.5 studs. Move them apart.",
                    offset_cir.name,
                    placement.position[0], placement.position[1], placement.position[2],
                    prev.name,
                ));
            }
        }

        placed_assets.push(PlacedAsset {
            name: offset_cir.name.clone(),
            bb_min,
            bb_max,
        });

        // Build the sub-model and add a unique name
        let mut sub_model = converter::cir_to_model_json(&offset_cir);
        let count = name_counts.entry(offset_cir.name.clone()).or_insert(0);
        *count += 1;
        let display_name = if *count > 1 {
            format!("{}_{}", offset_cir.name, count)
        } else {
            offset_cir.name.clone()
        };
        if let Value::Object(ref mut obj) = sub_model {
            obj.insert("name".into(), json!(display_name));
        }
        scene_children.push(sub_model);

        total_parts += offset_cir.parts.len();
        for axis in 0..3 {
            scene_min[axis] = scene_min[axis].min(bb_min[axis]);
            scene_max[axis] = scene_max[axis].max(bb_max[axis]);
        }
    }

    // Handle empty scene
    if total_parts == 0 {
        scene_min = [0.0; 3];
        scene_max = [0.0; 3];
    }

    let scene = json!({
        "className": "Model",
        "children": scene_children
    });

    // Write to disk
    let json_str = serde_json::to_string_pretty(&scene)?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, json_str)?;

    Ok(AssemblyResult {
        path: output_path.display().to_string(),
        total_parts,
        bounding_box_min: scene_min,
        bounding_box_max: scene_max,
        collision_warnings,
    })
}

fn resolve_cir(placement: &AssetPlacement, library_dir: &Path) -> anyhow::Result<Cir> {
    if let Some(ref name) = placement.asset_ref {
        let index = asset_library::load_index(library_dir)?;
        let entry = index
            .assets
            .iter()
            .find(|a| a.name == *name)
            .ok_or_else(|| anyhow::anyhow!("Asset '{}' not found in library", name))?;
        asset_library::load_asset_cir(library_dir, entry)
    } else if let Some(ref cir) = placement.cir {
        Ok(cir.clone())
    } else {
        Err(anyhow::anyhow!(
            "Asset placement must have either 'ref' or 'cir'"
        ))
    }
}
