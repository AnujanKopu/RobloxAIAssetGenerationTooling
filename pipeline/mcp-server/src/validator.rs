use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::cir::{Cir, CirPart};
use crate::scale_config::ScaleConfig;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub check: String,
    pub severity: String,
    pub part: Option<String>,
    pub message: String,
    pub fix: Option<String>,
}

pub struct ValidateOptions<'a> {
    pub scale: &'a ScaleConfig,
    pub palette: Option<&'a HashMap<String, [f64; 3]>>,
    pub max_parts: Option<usize>,
}

/// Compute the rotation-aware axis-aligned bounding box half-extents for a part.
/// Takes the part's local size and Euler rotation (degrees) and returns the
/// AABB half-extents [hx, hy, hz] after rotation.
fn rotated_half_extents(size: &[f64; 3], rot_deg: &[f64; 3]) -> [f64; 3] {
    let rx = rot_deg[0].to_radians();
    let ry = rot_deg[1].to_radians();
    let rz = rot_deg[2].to_radians();

    // Build rotation matrix columns (ZYX Euler order)
    let (sx, cx) = (rx.sin(), rx.cos());
    let (sy, cy) = (ry.sin(), ry.cos());
    let (sz, cz) = (rz.sin(), rz.cos());

    // Rotation matrix R = Rz * Ry * Rx
    let m = [
        [cy * cz, cz * sx * sy - cx * sz, cx * cz * sy + sx * sz],
        [cy * sz, cx * cz + sx * sy * sz, cx * sy * sz - cz * sx],
        [-sy,     cy * sx,                cx * cy               ],
    ];

    let hx = size[0] / 2.0;
    let hy = size[1] / 2.0;
    let hz = size[2] / 2.0;

    // AABB half-extent for each world axis = sum of |M[row][col]| * local_half[col]
    [
        (m[0][0].abs() * hx + m[0][1].abs() * hy + m[0][2].abs() * hz),
        (m[1][0].abs() * hx + m[1][1].abs() * hy + m[1][2].abs() * hz),
        (m[2][0].abs() * hx + m[2][1].abs() * hy + m[2][2].abs() * hz),
    ]
}

/// Get the AABB min/max for a part accounting for rotation
fn part_aabb(part: &CirPart) -> ([f64; 3], [f64; 3]) {
    let he = rotated_half_extents(&part.s, &part.r);
    (
        [part.p[0] - he[0], part.p[1] - he[1], part.p[2] - he[2]],
        [part.p[0] + he[0], part.p[1] + he[1], part.p[2] + he[2]],
    )
}

/// Calculate the overlap depth on each axis between two parts (rotation-aware).
/// Returns None if they don't overlap, or Some([dx, dy, dz]) with per-axis penetration.
fn overlap_depths(a: &CirPart, b: &CirPart) -> Option<[f64; 3]> {
    let (a_min, a_max) = part_aabb(a);
    let (b_min, b_max) = part_aabb(b);

    let mut depths = [0.0f64; 3];
    for axis in 0..3 {
        let depth = a_max[axis].min(b_max[axis]) - a_min[axis].max(b_min[axis]);
        if depth <= 0.05 {
            return None; // separated or just touching (within 0.05 tolerance)
        }
        depths[axis] = depth;
    }
    Some(depths)
}

pub fn validate(cir: &Cir, opts: &ValidateOptions) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    check_empty(cir, &mut errors);
    check_underground(cir, &mut errors);
    check_overlap_severity(cir, &mut errors, &mut warnings);
    check_floating(cir, &mut warnings);
    check_scale_violations(cir, opts.scale, &mut errors);
    check_extreme_dimensions(cir, opts.scale, &mut errors);
    check_part_budget(cir, opts.max_parts.unwrap_or(30), &mut warnings);
    check_palette(cir, opts.palette, &mut warnings);
    check_name_collisions(cir, &mut errors);

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

// --- Check 1: Empty model ---
fn check_empty(cir: &Cir, errors: &mut Vec<ValidationIssue>) {
    if cir.parts.is_empty() {
        errors.push(ValidationIssue {
            check: "empty_model".into(),
            severity: "error".into(),
            part: None,
            message: "Model has no parts".into(),
            fix: Some("Add at least one part to the model".into()),
        });
    }
}

// --- Check 2: Underground parts (rotation-aware) ---
fn check_underground(cir: &Cir, errors: &mut Vec<ValidationIssue>) {
    for part in &cir.parts {
        let (aabb_min, _) = part_aabb(part);
        let bottom_y = aabb_min[1];
        if bottom_y < -0.1 {
            errors.push(ValidationIssue {
                check: "underground".into(),
                severity: "error".into(),
                part: Some(part.n.clone()),
                message: format!("Part bottom at Y={:.2}, below ground", bottom_y),
                fix: Some(format!(
                    "Raise position Y by {:.2}",
                    (-bottom_y).max(0.0)
                )),
            });
        }
    }
}

// --- Check 3: Overlap with severity (rotation-aware) ---
// Seam overlaps (< 0.5 studs min penetration) → warning (normal for low-poly construction)
// Significant overlaps (≥ 0.5 studs min penetration) → ERROR (blocks conversion)
fn check_overlap_severity(
    cir: &Cir,
    errors: &mut Vec<ValidationIssue>,
    warnings: &mut Vec<ValidationIssue>,
) {
    // Threshold: if the minimum overlap axis (thinnest penetration) >= this, it's an error.
    // 1.0 stud allows wall corners (0.5 overlap), feature blocks on surfaces, roof seams.
    // Catches genuine clipping where parts occupy the same space by 1+ studs on all axes.
    let error_threshold = 1.0;

    for i in 0..cir.parts.len() {
        for j in (i + 1)..cir.parts.len() {
            if let Some(depths) = overlap_depths(&cir.parts[i], &cir.parts[j]) {
                let min_depth = depths[0].min(depths[1]).min(depths[2]);
                let max_depth = depths[0].max(depths[1]).max(depths[2]);

                if min_depth >= error_threshold {
                    errors.push(ValidationIssue {
                        check: "overlap".into(),
                        severity: "error".into(),
                        part: Some(format!("{}, {}", cir.parts[i].n, cir.parts[j].n)),
                        message: format!(
                            "Parts '{}' and '{}' significantly overlap (penetration: {:.1}×{:.1}×{:.1} studs, min={:.1})",
                            cir.parts[i].n, cir.parts[j].n,
                            depths[0], depths[1], depths[2], min_depth
                        ),
                        fix: Some(format!(
                            "Move '{}' so it does not clip into '{}'. Reduce overlap to < {:.1} studs on at least one axis.",
                            cir.parts[j].n, cir.parts[i].n, error_threshold
                        )),
                    });
                } else if max_depth > 0.1 {
                    warnings.push(ValidationIssue {
                        check: "overlap".into(),
                        severity: "warning".into(),
                        part: Some(format!("{}, {}", cir.parts[i].n, cir.parts[j].n)),
                        message: format!(
                            "Parts '{}' and '{}' have minor seam overlap ({:.2} studs)",
                            cir.parts[i].n, cir.parts[j].n, min_depth
                        ),
                        fix: None,
                    });
                }
            }
        }
    }
}

// --- Check 4: Floating parts (rotation-aware) ---
fn check_floating(cir: &Cir, warnings: &mut Vec<ValidationIssue>) {
    for (i, part) in cir.parts.iter().enumerate() {
        let (aabb_min, _) = part_aabb(part);
        let bottom_y = aabb_min[1];
        let on_ground = bottom_y.abs() < 0.2;

        if !on_ground {
            let touches_another = cir.parts.iter().enumerate().any(|(j, other)| {
                i != j && aabb_touching_rotated(part, other)
            });

            if !touches_another {
                warnings.push(ValidationIssue {
                    check: "floating".into(),
                    severity: "warning".into(),
                    part: Some(part.n.clone()),
                    message: format!(
                        "Part '{}' at Y={:.1} is floating (not on ground or touching another part)",
                        part.n, part.p[1]
                    ),
                    fix: Some(format!(
                        "Set position Y to {:.1} to rest on ground, or position adjacent to another part",
                        part.s[1] / 2.0
                    )),
                });
            }
        }
    }
}

// --- Check 5: Scale violations ---
fn check_scale_violations(cir: &Cir, scale: &ScaleConfig, errors: &mut Vec<ValidationIssue>) {
    for part in &cir.parts {
        let max_dim = part.s[0].max(part.s[1]).max(part.s[2]);
        if max_dim > scale.max_single_part_axis {
            errors.push(ValidationIssue {
                check: "scale_violation".into(),
                severity: "error".into(),
                part: Some(part.n.clone()),
                message: format!(
                    "Part '{}' has dimension {:.1}, exceeds max {:.1}",
                    part.n, max_dim, scale.max_single_part_axis
                ),
                fix: Some(format!(
                    "Reduce largest dimension to under {:.1} studs",
                    scale.max_single_part_axis
                )),
            });
        }
    }
}

// --- Check 6: Extreme dimensions ---
fn check_extreme_dimensions(cir: &Cir, scale: &ScaleConfig, errors: &mut Vec<ValidationIssue>) {
    let labels = ["X", "Y", "Z"];
    for part in &cir.parts {
        for axis in 0..3 {
            if part.s[axis] < scale.min_single_part_axis {
                errors.push(ValidationIssue {
                    check: "extreme_dimension".into(),
                    severity: "error".into(),
                    part: Some(part.n.clone()),
                    message: format!(
                        "Part '{}' {} size {:.2} below minimum {:.2}",
                        part.n, labels[axis], part.s[axis], scale.min_single_part_axis
                    ),
                    fix: Some(format!(
                        "Set {} size to at least {:.2}",
                        labels[axis], scale.min_single_part_axis
                    )),
                });
            }
        }
    }
}

// --- Check 7: Part count budget ---
fn check_part_budget(cir: &Cir, max: usize, warnings: &mut Vec<ValidationIssue>) {
    if cir.parts.len() > max {
        warnings.push(ValidationIssue {
            check: "part_budget".into(),
            severity: "warning".into(),
            part: None,
            message: format!(
                "Model has {} parts, exceeds budget of {}",
                cir.parts.len(),
                max
            ),
            fix: Some(format!(
                "Reduce to {} parts or fewer for low-poly style",
                max
            )),
        });
    }
}

// --- Check 8: Color palette ---
fn check_palette(
    cir: &Cir,
    palette: Option<&HashMap<String, [f64; 3]>>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let palette = match palette {
        Some(p) => p,
        None => return,
    };

    let palette_colors: Vec<&[f64; 3]> = palette.values().collect();

    for part in &cir.parts {
        let in_palette = palette_colors.iter().any(|pc| {
            (pc[0] - part.c[0]).abs() < 0.05
                && (pc[1] - part.c[1]).abs() < 0.05
                && (pc[2] - part.c[2]).abs() < 0.05
        });
        if !in_palette {
            warnings.push(ValidationIssue {
                check: "palette".into(),
                severity: "warning".into(),
                part: Some(part.n.clone()),
                message: format!(
                    "Part '{}' color [{:.2},{:.2},{:.2}] not in declared palette",
                    part.n, part.c[0], part.c[1], part.c[2]
                ),
                fix: Some("Use a color from the palette".into()),
            });
        }
    }
}

// --- Check 9: Name collisions ---
fn check_name_collisions(cir: &Cir, errors: &mut Vec<ValidationIssue>) {
    let mut seen = HashSet::new();
    for part in &cir.parts {
        if !seen.insert(&part.n) {
            errors.push(ValidationIssue {
                check: "name_collision".into(),
                severity: "error".into(),
                part: Some(part.n.clone()),
                message: format!("Duplicate part name '{}'", part.n),
                fix: Some(format!(
                    "Rename one of the '{}' parts to a unique name",
                    part.n
                )),
            });
        }
    }
}

/// AABB touching (rotation-aware) — true if two parts are within tolerance of each other
fn aabb_touching_rotated(a: &CirPart, b: &CirPart) -> bool {
    let tolerance = 0.3;
    let (a_min, a_max) = part_aabb(a);
    let (b_min, b_max) = part_aabb(b);
    for axis in 0..3 {
        if (a_max[axis] + tolerance) < b_min[axis] || (b_max[axis] + tolerance) < a_min[axis] {
            return false;
        }
    }
    true
}

// ── Public helpers for use by assembler ──

/// Compute the AABB [min, max] for a CIR model, accounting for part rotation.
pub fn model_aabb(cir: &Cir) -> ([f64; 3], [f64; 3]) {
    if cir.parts.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = [f64::MAX; 3];
    let mut mx = [f64::MIN; 3];
    for part in &cir.parts {
        let (lo, hi) = part_aabb(part);
        for axis in 0..3 {
            mn[axis] = mn[axis].min(lo[axis]);
            mx[axis] = mx[axis].max(hi[axis]);
        }
    }
    (mn, mx)
}

/// Check if two AABBs (given as [min, max]) overlap by more than `margin` studs on all axes.
pub fn aabb_boxes_overlap(
    a_min: &[f64; 3], a_max: &[f64; 3],
    b_min: &[f64; 3], b_max: &[f64; 3],
    margin: f64,
) -> bool {
    for axis in 0..3 {
        let depth = a_max[axis].min(b_max[axis]) - a_min[axis].max(b_min[axis]);
        if depth <= margin {
            return false;
        }
    }
    true
}
