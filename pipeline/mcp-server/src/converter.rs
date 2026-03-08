use serde_json::{json, Map, Value};
use std::path::Path;

use crate::cir::{Cir, CirPart};

/// Map CIR part type to Roblox ClassName
fn class_name_for_type(part_type: &str) -> &str {
    match part_type {
        "wedge" => "WedgePart",
        "corner_wedge" => "CornerWedgePart",
        // cylinder and ball use Part with a Shape property
        _ => "Part",
    }
}

/// Build the $properties map for a single CIR part
fn build_properties(part: &CirPart) -> Map<String, Value> {
    let mut props = Map::new();

    props.insert("Anchored".into(), json!(true));
    props.insert("Locked".into(), json!(true));
    props.insert("Position".into(), json!(part.p));
    props.insert("Size".into(), json!(part.s));
    props.insert("Color".into(), json!(part.c));
    props.insert("Material".into(), json!(part.m));
    props.insert("TopSurface".into(), json!("Smooth"));
    props.insert("BottomSurface".into(), json!("Smooth"));

    // Rotation — only include if non-zero
    if part.r.iter().any(|v| v.abs() > 0.001) {
        props.insert("Orientation".into(), json!(part.r));
    }

    // Shape enum for cylinder and ball (these use the Part class)
    match part.t.as_str() {
        "cylinder" => {
            props.insert("Shape".into(), json!("Cylinder"));
        }
        "ball" => {
            props.insert("Shape".into(), json!("Ball"));
        }
        _ => {}
    }

    props
}

/// Convert a full CIR model to a Rojo .model.json Value
pub fn cir_to_model_json(cir: &Cir) -> Value {
    let children: Vec<Value> = cir
        .parts
        .iter()
        .map(|part| {
            json!({
                "name": part.n,
                "className": class_name_for_type(&part.t),
                "properties": build_properties(part)
            })
        })
        .collect();

    json!({
        "className": "Model",
        "children": children
    })
}

/// Convert CIR to .model.json and write to disk
pub fn write_model_json(cir: &Cir, output_path: &Path) -> anyhow::Result<String> {
    let model = cir_to_model_json(cir);
    let json_str = serde_json::to_string_pretty(&model)?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(output_path, &json_str)?;
    Ok(output_path.display().to_string())
}

/// Compute the axis-aligned bounding box of a CIR model
pub fn compute_bounding_box(cir: &Cir) -> ([f64; 3], [f64; 3]) {
    if cir.parts.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }

    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];

    for part in &cir.parts {
        for axis in 0..3 {
            let lo = part.p[axis] - part.s[axis] / 2.0;
            let hi = part.p[axis] + part.s[axis] / 2.0;
            min[axis] = min[axis].min(lo);
            max[axis] = max[axis].max(hi);
        }
    }

    (min, max)
}
