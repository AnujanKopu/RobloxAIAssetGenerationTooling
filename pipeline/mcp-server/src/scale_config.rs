use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConfig {
    pub humanoid_height: f64,
    pub door_height: f64,
    pub door_width: f64,
    pub window_height: f64,
    pub window_width: f64,
    pub fence_height: f64,
    pub fence_post_width: f64,
    pub wall_thickness: f64,
    pub floor_unit: f64,
    pub roof_overhang: f64,
    pub tree_trunk_width_range: [f64; 2],
    pub tree_height_range: [f64; 2],
    pub max_single_part_axis: f64,
    pub min_single_part_axis: f64,
}

pub fn load_scale_config(config_dir: &Path) -> anyhow::Result<ScaleConfig> {
    let path = config_dir.join("scale.json");
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn load_palette(config_dir: &Path, name: &str) -> anyhow::Result<HashMap<String, [f64; 3]>> {
    let path = config_dir.join("palette.json");
    let content = std::fs::read_to_string(&path)?;
    let palettes: HashMap<String, HashMap<String, [f64; 3]>> = serde_json::from_str(&content)?;
    palettes
        .get(name)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Palette '{}' not found. Available: {:?}",
                name,
                palettes.keys().collect::<Vec<_>>()
            )
        })
}

pub fn list_palettes(config_dir: &Path) -> anyhow::Result<Vec<String>> {
    let path = config_dir.join("palette.json");
    let content = std::fs::read_to_string(&path)?;
    let palettes: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
    Ok(palettes.keys().cloned().collect())
}
